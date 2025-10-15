/*
  Warnings:

  - You are about to alter the column `argNonce` on the `ExecutedMessage` table. The data in that column could be lost. The data in that column will be cast from `Int` to `BigInt`.
  - You are about to alter the column `txTime` on the `ExecutedMessage` table. The data in that column could be lost. The data in that column will be cast from `String` to `Int`.
  - You are about to alter the column `argSlot` on the `HeadUpdate` table. The data in that column could be lost. The data in that column will be cast from `Int` to `BigInt`.
  - You are about to alter the column `txTime` on the `HeadUpdate` table. The data in that column could be lost. The data in that column will be cast from `String` to `Int`.
  - You are about to alter the column `argNonce` on the `SentMessage` table. The data in that column could be lost. The data in that column will be cast from `Int` to `BigInt`.
  - You are about to alter the column `txTime` on the `SentMessage` table. The data in that column could be lost. The data in that column will be cast from `String` to `Int`.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_ExecutedMessage" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argNonce" BIGINT NOT NULL,
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
    "txTime" INTEGER NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_ExecutedMessage" ("argMessage", "argMessageRoot", "argNonce", "argStatus", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime") SELECT "argMessage", "argMessageRoot", "argNonce", "argStatus", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime" FROM "ExecutedMessage";
DROP TABLE "ExecutedMessage";
ALTER TABLE "new_ExecutedMessage" RENAME TO "ExecutedMessage";
CREATE TABLE "new_HeadUpdate" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argSlot" BIGINT NOT NULL,
    "argRoot" TEXT NOT NULL,
    "augBlockForSlot" INTEGER NOT NULL,
    "chainId" INTEGER NOT NULL,
    "contractAddress" TEXT NOT NULL,
    "txHash" TEXT NOT NULL,
    "txIndex" INTEGER NOT NULL,
    "logIndex" INTEGER NOT NULL,
    "txBlockNumber" INTEGER NOT NULL,
    "txBlockHash" TEXT NOT NULL,
    "txTime" INTEGER NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_HeadUpdate" ("argRoot", "argSlot", "augBlockForSlot", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime") SELECT "argRoot", "argSlot", "augBlockForSlot", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime" FROM "HeadUpdate";
DROP TABLE "HeadUpdate";
ALTER TABLE "new_HeadUpdate" RENAME TO "HeadUpdate";
CREATE TABLE "new_SentMessage" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "argNonce" BIGINT NOT NULL,
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
    "txTime" INTEGER NOT NULL,
    "reorged" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_SentMessage" ("argMessage", "argMessageRoot", "argNonce", "augData", "augGasLimit", "augMsgSender", "augRecipient", "augRecipientChainId", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime") SELECT "argMessage", "argMessageRoot", "argNonce", "augData", "augGasLimit", "augMsgSender", "augRecipient", "augRecipientChainId", "chainId", "contractAddress", "createdAt", "id", "logIndex", "reorged", "txBlockHash", "txBlockNumber", "txHash", "txIndex", "txTime" FROM "SentMessage";
DROP TABLE "SentMessage";
ALTER TABLE "new_SentMessage" RENAME TO "SentMessage";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
