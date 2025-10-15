/**
 * Sentry logging for Node.js + TypeScript
 *
 * To be used within the Integration Client
 **/

import * as Sentry from '@sentry/node';
import '@sentry/tracing';

export type SentryConfig = {
    dsn: string;
};

class SentryClient {
    config?: SentryConfig;

    constructor(config?: SentryConfig) {
        this.config = config;
    }

    init() {
        Sentry.init({
            dsn: this?.config?.dsn
        });

        return this;
    }
}

export default SentryClient;
