import { ethers, ContractTransaction, ContractReceipt } from 'ethers';
import { exit } from 'process';

export function safeWaitForTx(
    tx: ContractTransaction,
    methodName: string,
    contractName: string,
    logger: any
) {
    tx.wait()
        .then((receipt: ContractReceipt) => {
            logger.info(
                `Successful ${methodName} on ${contractName} with tx=${receipt.transactionHash} and gas=${receipt.gasUsed}`
            );
        })
        .catch((error: any) => {
            if (error.code === ethers.errors.CALL_EXCEPTION) {
                logger.error(
                    `Failed to ${methodName} ${contractName} due to call exception for tx=${error.transactionHash}`
                );
            } else if (error.code === ethers.errors.TRANSACTION_REPLACED) {
                logger.error(
                    `Failed to ${methodName} ${contractName} with reason=${error.reason} and new tx=${error.replacement}`
                );
            } else {
                logger.error('Weird error was encountered while waiting for transaction.');
                console.error(error);
                exit();
            }
        });
}
