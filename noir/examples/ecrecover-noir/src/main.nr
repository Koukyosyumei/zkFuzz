use dep::std;
use dep::keccak256;

unconstrained
fn u8_32_to_u8_64_unconstrained(
    arr_a: [u8; 32],
    arr_b: [u8; 32],
) -> [u8; 64] {
    let mut combined: [u8; 64] = [0; 64];

    for i in 0..32 {
        combined[i] = arr_a[i];
    }
    for i in 0..32 {
        combined[i + 32] = arr_b[i];
    }

    combined
}

unconstrained
fn split_u8_64_unconstrained(
    arr: [u8; 64]
) -> ([u8; 32], [u8; 32]) {
    let mut arr_a: [u8; 32] = [0; 32];
    let mut arr_b: [u8; 32] = [0; 32];

    for i in 0..32 {
        arr_a[i] = arr[i];
        arr_b[i] = arr[i + 32];
    };

    (arr_a, arr_b)
}

pub struct PubKey {
    pub_x: [u8; 32],
    pub_y: [u8; 32],
}

unconstrained
fn split_uncompressed_pub_key(
    pub_key: [u8; 65]
) -> ([u8; 32], [u8; 32]) {
    let mut pub_key_x: [u8; 32] = [0; 32];
    let mut pub_key_y: [u8; 32] = [0; 32];

    for i in 0..32 {
        pub_key_x[i] = pub_key[i + 1];
        pub_key_y[i] = pub_key[i + 32 + 1];
    };

    (pub_key_x, pub_key_y)
}

impl PubKey {
    fn from_xy(pub_x: [u8; 32], pub_y: [u8; 32]) -> PubKey {
        PubKey {
            pub_x,
            pub_y,
        }
    }

    fn from_unified(pub_key: [u8; 64]) -> PubKey {
        let (key_x, key_y) = unsafe{ split_u8_64_unconstrained(pub_key) };

        PubKey {
            pub_x: key_x,
            pub_y: key_y,
        }
    }

    fn from_uncompressed(pub_key: [u8; 65]) -> PubKey {
        assert(pub_key[0] == 0x04);
        let (key_x, key_y) = unsafe{ split_uncompressed_pub_key(pub_key) };

        PubKey {
            pub_x: key_x,
            pub_y: key_y,
        }
    }

    fn verify_sig(self, signature: [u8; 64], hashed_message: [u8; 32]) -> bool {
        std::ecdsa_secp256k1::verify_signature(self.pub_x, self.pub_y, signature, hashed_message)
    }

    fn to_eth_address(self) -> Field {
        let pub_key = unsafe{ u8_32_to_u8_64_unconstrained(self.pub_x, self.pub_y) };
        let hashed_pub_key = keccak256::keccak256(pub_key, 64);

        let mut addr: Field = 0;
        for i in 0..20 {
            // shift left by 8 and add the new value
            addr = (addr * 256) + hashed_pub_key[i + 12] as Field;
        }

        addr
    }

    fn ecrecover(
        self,
        signature: [u8; 64],
        hashed_message: [u8; 32]
    ) -> Field {
        assert(self.verify_sig(signature, hashed_message));

        self.to_eth_address()
    }
}


pub fn main(
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64], // clip v value
    hashed_message: [u8; 32]
) -> pub Field {
    let key = PubKey::from_xy(pub_key_x, pub_key_y);

    assert(key.verify_sig(signature, hashed_message));
    let addr = key.to_eth_address();

    addr
}

#[test]
fn test_ecrecover() {
    let pub_key_x = [131, 24, 83, 91, 84, 16, 93, 74, 122, 174, 96, 192, 143, 196, 95, 150, 135, 24, 27, 79, 223, 198, 37, 189, 26, 117, 63, 167, 57, 127, 237, 117];
    let pub_key_y = [53, 71, 241, 28, 168, 105, 102, 70, 242, 243, 172, 176, 142, 49, 1, 106, 250, 194, 62, 99, 12, 93, 17, 245, 159, 97, 254, 245, 123, 13, 42, 165];
    let signature = [57, 17, 112, 239, 241, 30, 64, 157, 170, 50, 85, 145, 156, 69, 226, 85, 147, 164, 10, 82, 71, 93, 42, 132, 200, 220, 161, 255, 95, 241, 211, 141, 81, 7, 150, 25, 25, 27, 162, 213, 80, 61, 12, 170, 50, 4, 154, 203, 252, 229, 119, 29, 202, 153, 50, 25, 126, 145, 245, 23, 136, 75, 29, 177];
    let hashed_message = [13, 82, 120, 60, 76, 186, 215, 235, 175, 126, 185, 67, 252, 100, 143, 82, 130, 165, 32, 112, 68, 47, 193, 141, 141, 209, 109, 219, 47, 203, 175, 102];

    let key = PubKey::from_xy(pub_key_x, pub_key_y);

    assert(key.verify_sig(signature, hashed_message));
    let addr = key.to_eth_address();
    assert(addr == 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266);
}