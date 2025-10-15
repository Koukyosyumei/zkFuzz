# telepathy

building towards a proof-based future, starting with proof of consensus ✨

### Installation

We require node 16.15.0 and yarn 3.
https://yarnpkg.com/getting-started/install

```
corepack enable
npm i -g corepack
corepack prepare yarn@stable --activate
```

To install dependencies, use

```
yarn
```

You will need `circom-stable`

-   https://github.com/jtguibas/circom-stable
    git clone https://github.com/jtguibas/circom-stable
    `cargo install --path circom`

### Troubleshooting

Make sure to run `yarn` at the top-level, before running `yarn` in each subdir, otherwise there are issues. If you didn't do this, `rm -rf node_modules` in the top-level folder, `rm yarn.lock` and reinstall everything by running `yarn` in the top-level folder, followed by running `yarn` in each subdirectory.

If there are issues with yarn, you can try running:

-   `yarn dedupe`
-   `yarn dedupe react-dom`

If you get an error "Cannot write file ... because it would overwrite input file.", then do the following:

-   `rm -rf dist/`
-   `yarn cache clean`
-   `yarn`

### TODO

-   Make sure before running the frontend that you run `cd cli; npx tsx src/frontend.ts` to generate all the requisite public folder files for the frontend. In the future, this should be moved to the `frontend` folder as a post-install hook.

```
rm -rf node_modules && rm -rf **/node_modules && rm yarn.lock && rm **/tsconfig.tsbuildinfo && rm -rf **/dist && yarn cache clean
```

### Fresh install

-   Make sure you are using nvm 16.5.0
-   Make sure you are using yarn3 with the corepack instructions
-   Make sure you have forge / foundry
-   circom stable 2.5.0

-   `cd contracts; forge install; forge test; forge build --extra-output-files abi --out abi;` to make sure solidity is working
-   `yarn` at top-level
-   `cd sdk; yarn typechain`

To deploy the system

-   `cd cli; yarn lightclient deploy;`
-   `yarn counter deploy;`
-   `yarn counter verify`
-   `yarn bridge deploy`
-   `yarn bridge verify`

To run the frontend
`cd cli; yarn tsx src/frontend.ts`
