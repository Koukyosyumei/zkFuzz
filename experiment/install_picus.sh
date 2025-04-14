#!/bin/bash

sudo apt-get update
sudo apt-get install libgmp-dev

pip install z3-solver

echo "Cloning CVC4 repository..."
git clone https://github.com/alex-ozdemir/CVC4.git
cd CVC4
echo "Checking out branch 'ff'..."
git checkout ff
cd ..

echo "Downloading CoCoALib (0.99800)..."
wget https://cocoa.altervista.org/cocoalib/tgz/CoCoALib-0.99800.tgz
tar -xzf CoCoALib-0.99800.tgz
cd CoCoALib-0.99800
./configure
make -j4
sudo make install
cd ..

cd CVC4
echo "Running CVC4 configure script (first pass)..."
./configure.sh --cocoa --auto-download
cd ..

mv ./CoCoALib-0.99800/ ./a
cd ./a
echo "Applying patch..."
patch -p1 < ../CVC4/cmake/deps-utils/CoCoALib-0.99800-trace.patch
cd ..
mv ./a ./CoCoALib-0.99800/

cd ./CoCoALib-0.99800/
echo "Following CoCoALib installation instructions..."
./configure
make -j4
yes | sudo make install
cd ..

echo "Cloning libpoly repository..."
git clone https://github.com/SRI-CSL/libpoly.git
cd libpoly
echo "Checking out latest development version (not a release)..."
echo "Building and installing libpoly..."
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr/local
make -j4
sudo make install
cd ..

cd CVC4
./configure.sh --cocoa --auto-download
cd build
sudo make -j4 install
cd ../..

wget https://github.com/iden3/circom/releases/download/v2.2.0/circom-linux-amd64
sudo chmod 777 ./circom-linux-amd64
sudo cp ./circom-linux-amd64 /usr/local/bin/circom

wget https://download.racket-lang.org/installers/8.14/racket-8.14-x86_64-linux-cs.sh
{ echo no; echo 1; } | sudo sh ./racket-8.14-x86_64-linux-cs.sh

sudo apt install libgmp-dev
pip3 install tomli scikit-build Cython
