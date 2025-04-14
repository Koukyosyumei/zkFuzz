pragma circom 2.1.8;

include "../circomlib-cff5ab6/bitify.circom";
include "../circomlib-cff5ab6/comparators.circom";
include "../circomlib-cff5ab6/mimcsponge.circom";
include "../circomlib-cff5ab6/switcher.circom";

template LayoutEligibility(MAP_WIDTH, MAP_HEIGHT, MAX_HOUSE, MAX_HOUSE_WIDTH, MAX_HOUSE_HEIGHT, MAX_PATH_SIZE) {
  var MAP_SIZE = MAP_WIDTH * MAP_HEIGHT;
  var MAX_HOUSE_SIZE = MAX_HOUSE_WIDTH * MAX_HOUSE_HEIGHT;
  var MAX_TYPE = 5;
  var MAX_PATH_HOUSE_SIZE = (MAX_HOUSE_WIDTH + 2) * (MAX_HOUSE_HEIGHT + 2);

  // witness
  signal input layoutPlacements[MAP_SIZE];
  signal input path[MAX_PATH_SIZE];
  signal input pathMask[MAX_PATH_SIZE];

  // public output
  signal output rate;
  signal output storage;

  signal typeAccumulation[MAX_TYPE][MAP_SIZE];
  signal typeZero[MAX_TYPE][MAP_SIZE];
  signal accum[MAP_SIZE][MAX_TYPE][MAX_HOUSE_SIZE];
  signal layoutZero[MAP_SIZE][MAX_TYPE][MAX_HOUSE_SIZE];
  signal accumZero[MAP_SIZE][MAX_TYPE];
  signal accumTemp[MAP_SIZE][MAX_TYPE];
  // to verify path
  signal accumProduct[MAP_SIZE][MAX_PATH_SIZE + 1];
  signal accumProductTemp[MAP_SIZE][MAX_PATH_SIZE];
  signal accumProductTempTemp[MAP_SIZE][MAX_PATH_SIZE];
  signal selectorPath[MAP_SIZE];

  // to verify path is valid
  signal accumProductPath[MAX_PATH_SIZE][5];
  signal accumProductPathTemp[MAX_PATH_SIZE][4];

  // to verify the road of layoutPlacements is connect to factory
  signal accumProductZero[MAP_SIZE][MAX_TYPE][MAX_PATH_HOUSE_SIZE];
  signal layoutProduct[MAP_SIZE][MAX_TYPE][MAX_PATH_HOUSE_SIZE];
  signal productZero[MAP_SIZE][MAX_TYPE];
  signal productTemp[MAP_SIZE][MAX_TYPE];

  // to verify pathMask
  signal accumPathMaskZero[MAX_PATH_SIZE];

  signal type0num;
  signal type1num;
  signal type2num;
  signal type3num;
  signal type4num;

  // house space mask pattern
  var pattern[MAX_TYPE][MAX_HOUSE_SIZE];
  pattern[0] = [1,1,0,0,1,1,0,0,0,0,0,0];
  pattern[1] = [1,1,1,1,1,1,1,1,0,0,0,0];
  pattern[2] = [1,0,0,0,1,0,0,0,0,0,0,0];
  pattern[3] = [1,1,0,0,1,1,0,0,0,0,0,0];
  pattern[4] = [1,1,1,0,1,1,1,0,1,1,1,0];

  // path house space mask pattern
  var pattern_path[MAX_TYPE][MAX_PATH_HOUSE_SIZE];
  pattern_path[0] = [7,1,1,7,7,7,1,7,7,1,7,7,1,7,7,1,7,7,7,1,1,7,7,7,7,7,7,7,7,7];
  pattern_path[1] = [7,1,1,1,1,7,1,7,7,7,7,1,1,7,7,7,7,1,7,1,1,1,1,7,7,7,7,7,7,7];
  pattern_path[2] = [7,1,7,7,7,7,1,7,1,7,7,7,1,7,1,7,7,7,7,1,7,7,7,7,7,7,7,7,7,7];
  pattern_path[3] = [7,1,1,7,7,7,1,7,7,1,7,7,1,7,7,1,7,7,7,1,1,7,7,7,7,7,7,7,7,7];
  pattern_path[4] = [7,1,1,1,7,7,1,7,7,7,1,7,1,7,7,7,1,7,1,7,7,7,1,7,7,1,1,1,7,7];

  // verfiy pathMask is valid
  pathMask[0] === 1; // constraint the first element is 1
  accumPathMaskZero[0] <== 0;
  for (var i = 1; i < MAX_PATH_SIZE; i++) {
    // constraint every element is zero or one
    pathMask[i] * (1 - pathMask[i]) === 0;
    // avoid [... 0 1 ...]
    accumPathMaskZero[i] <== accumPathMaskZero[i - 1] + (1 - pathMask[i - 1]) * pathMask[i];
  }
  accumPathMaskZero[MAX_PATH_SIZE - 1] === 0;

  for (var i = 0; i < MAP_SIZE; i++) {
    var x = i % MAP_WIDTH;
    var y = i \ MAP_WIDTH;

    // signal area[MAX_HOUSE_SIZE];
    // log("cell", i);
    for (var k = 0; k < MAX_TYPE; k++) {
      var num = k + 2;
      typeZero[k][i] <== IsZero()(layoutPlacements[i] - num);
      if (i == 0) {
        typeAccumulation[k][i] <== typeZero[k][i];
      } else {
        typeAccumulation[k][i] <== typeZero[k][i] + typeAccumulation[k][i - 1];
      }

      // log("type", k);
      // ignore the first one which is the type number
      accum[i][k][0] <== 0;
      for (var j = 1; j < MAX_HOUSE_SIZE; j++) {
        var ax = j % MAX_HOUSE_WIDTH;
        var ay = j \ MAX_HOUSE_WIDTH;
        // if layoutPlacements[i] != 0 then the cell must be 0
        var index = i + ax + ay * MAP_WIDTH;
        // log(index, ax, ay);
        if (index >= MAP_SIZE || index < 0) {
          layoutZero[i][k][j] <== 1;// TODO
        } else {
          layoutZero[i][k][j] <== IsZero()(layoutPlacements[index] * pattern[k][j]);
        }
        accum[i][k][j] <== layoutZero[i][k][j] + accum[i][k][j - 1];
        // log("step", accum[i][k][j]);
        // area[j] <== (x + ax < MAP_WIDTH && y + ay < MAP_HEIGHT)
        //   ? layoutPlacements[i + ax + ay * MAP_WIDTH]
        //   : 0;
      }

      // log("final", accum[i][k][MAX_HOUSE_SIZE - 1]);
      // if typeZero[k][i] === 1 then require accumZero[i][k] === 1
      // Truth table:
      // a = typeZero[k][i] , b = accumZero[i][k]
      //  a | b | Output
      // ---------------
      //  0 | 0 |   1
      //  0 | 1 |   1
      //  1 | 0 |   0
      //  1 | 1 |   1
      // P = 1 - a + ab;
      accumZero[i][k] <== IsZero()(accum[i][k][MAX_HOUSE_SIZE - 1] - MAX_HOUSE_SIZE + 1);
      accumTemp[i][k] <== 1 - typeZero[k][i] + typeZero[k][i] * accumZero[i][k];
      accumTemp[i][k] === 1;

      accumProductZero[i][k][0] <== 1;
      for (var l = 1; l < MAX_PATH_HOUSE_SIZE; l++) {
        var bx = l % (MAX_HOUSE_WIDTH + 2);
        var by = l \ (MAX_HOUSE_WIDTH + 2);
        var index_i = i + bx + by * MAP_WIDTH;
        if (index_i >= MAP_SIZE || index_i < 0) {
          layoutProduct[i][k][l] <== 0;
        } else {
          layoutProduct[i][k][l] <== layoutPlacements[index_i] - pattern_path[k][l];
        }
        accumProductZero[i][k][l] <== layoutProduct[i][k][l] * accumProductZero[i][k][l - 1];
        // log("accumProductZero[i][k][l]: ", accumProductZero[i][k][l]);
      }
      // log("accumProductZero[i][k][MAX_HOUSE_SIZE - 1]: ", accumProductZero[i][k][MAX_PATH_HOUSE_SIZE - 1]);
      productZero[i][k] <== IsZero()(accumProductZero[i][k][MAX_PATH_HOUSE_SIZE - 1]);
      // log("productZero: ", productZero[i][k]);
      productTemp[i][k] <== 1 - typeZero[k][i] + typeZero[k][i] * productZero[i][k];
      // log("productTemp: ", productTemp[i][k]);
      productTemp[i][k] === 1;
    }

    // verify path multisets is correct
    accumProduct[i][0] <== 1;
    for (var j = 0; j < MAX_PATH_SIZE; j++) {
      accumProductTempTemp[i][j] <== IsZero()(i - path[j]);
      accumProductTemp[i][j] <== accumProductTempTemp[i][j] * pathMask[j];
      // log("path[j]", path[j]);
      // log("accumProductTemp[i][j]:", accumProductTemp[i][j]);
      accumProduct[i][j + 1] <== accumProduct[i][j] * (1 - accumProductTemp[i][j]);
      // log("accumProduct[i][j + 1]:", accumProduct[i][j + 1]);
    }
    selectorPath[i] <== IsZero()(accumProduct[i][MAX_PATH_SIZE]);
    // log("selectorPath[i]:", selectorPath[i]);
    selectorPath[i] * (layoutPlacements[i] - 1) === 0;
  }

  // verify path is valid
  for (var j = 0; j < MAX_PATH_SIZE - 1; j++) {
    accumProductPath[j][0] <== 1;

    accumProductPathTemp[j][0] <== IsZero()(path[j] - path[j + 1] + 1);
    accumProductPath[j][1] <== accumProductPath[j][0] * accumProductPathTemp[j][0];

    accumProductPathTemp[j][1] <== IsZero()(path[j] - path[j + 1] - 1);
    accumProductPath[j][2] <== accumProductPath[j][1] * accumProductPathTemp[j][1];

    accumProductPathTemp[j][2] <== IsZero()(path[j] - path[j + 1] - MAP_WIDTH);
    accumProductPath[j][3] <== accumProductPath[j][2] * accumProductPathTemp[j][2];

    accumProductPathTemp[j][3] <== IsZero()(path[j] - path[j + 1] + MAP_WIDTH);
    accumProductPath[j][4] <== accumProductPath[j][3] * accumProductPathTemp[j][3];

    accumProductPath[j][4] * pathMask[j] === 0;
  }

  type0num <== typeAccumulation[0][MAP_SIZE - 1];
  type1num <== typeAccumulation[1][MAP_SIZE - 1];
  type2num <== typeAccumulation[2][MAP_SIZE - 1];
  type3num <== typeAccumulation[3][MAP_SIZE - 1];
  type4num <== typeAccumulation[4][MAP_SIZE - 1];

  signal out1 <== type0num;
  signal out2calc <-- out1;
  signal type1calc <== type1num;
  signal compare2 <-- type1calc * 2;
  type1calc * 2 === compare2;
  signal out2, t2;
  (out2, t2) <== Switcher()(LessThan(5)([out2calc, compare2]), out2calc, compare2);

  signal out3calc <== out2;
  signal type2cal <== type2num;
  signal compare3 <-- type2cal * 2;
  type2cal * 2 === compare3;
  signal out3, t3;
  (out3, t3) <== Switcher()(LessThan(5)([out3calc, compare3]), out3calc, compare3);

  rate <== out3;

  // at least one seaport
  signal seaport <== GreaterThan(5)([type3num, 0]);
  seaport === 1;

  // storage
  storage <== type4num;

}

// component main = LayoutEligibility(18, 8, 30, 4, 3, 144);
