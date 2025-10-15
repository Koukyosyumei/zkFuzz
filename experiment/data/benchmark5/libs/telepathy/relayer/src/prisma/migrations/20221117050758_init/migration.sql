-- CreateTable
CREATE TABLE "HeadUpdate" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argSlot" INTEGER NOT NULL,
    "argRoot" TEXT NOT NULL,
    "augBlockForSlot" INTEGER NOT NULL,
    "chainId" INTEGER NOT NULL,
    "contractAddress" TEXT NOT NULL,
    "txHash" TEXT NOT NULL,
    "txIndex" INTEGER NOT NULL,
    "logIndex" INTEGER NOT NULL,
    "txBlockNumber" INTEGER NOT NULL,
    "txBlockHash" TEXT NOT NULL,
    "txTime" TEXT NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);

-- CreateTable
CREATE TABLE "SentMessage" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argNonce" INTEGER NOT NULL,
    "argMessageRoot" TEXT NOT NULL,
    "argMessage" TEXT NOT NULL,
    "augMsgSender" TEXT NOT NULL,
    "augRecipient" TEXT NOT NULL,
    "augRecipientChainId" INTEGER NOT NULL,
    "augGasLimit" INTEGER NOT NULL,
    "augData" TEXT NOT NULL,
    "chainId" INTEGER NOT NULL,
    "contractAddress" TEXT NOT NULL,
    "txHash" TEXT NOT NULL,
    "txIndex" INTEGER NOT NULL,
    "logIndex" INTEGER NOT NULL,
    "txBlockNumber" INTEGER NOT NULL,
    "txBlockHash" TEXT NOT NULL,
    "txTime" TEXT NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);

-- CreateTable
CREATE TABLE "ExecutedMessage" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argNonce" INTEGER NOT NULL,
    "argMessageRoot" TEXT NOT NULL,
    "argMessage" TEXT NOT NULL,
    "argStatus" BOOLEAN NOT NULL,
    "chainId" INTEGER NOT NULL,
    "contractAddress" TEXT NOT NULL,
    "txHash" TEXT NOT NULL,
    "txIndex" INTEGER NOT NULL,
    "logIndex" INTEGER NOT NULL,
    "txBlockNumber" INTEGER NOT NULL,
    "txBlockHash" TEXT NOT NULL,
    "txTime" TEXT NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);
