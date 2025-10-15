/* eslint-disable */

export type UnparsedResponse = any;

// @ts-ignore
BigInt.prototype.toJSON = function () {
    return this.toString();
};

/* eslint-enable */
