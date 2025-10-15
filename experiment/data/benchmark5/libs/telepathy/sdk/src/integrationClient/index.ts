/**
 * This is an IntegrationClient object to be used across different applications to instantiate common artifacts such as logging, notifications etc
 *
 * Logger is enabled by _default_
 * Disabling the logger can be done by passing in { disableLogger: true }
 *
 * Sentry is disabled by _default_
 * Enabling requires setting Environtment Variable => SENTRY_DSN
 *  - OR -
 * Passing in a sentry config with dsn value { sentry: { dsn: <YOUR DSN> } }
 * **NOTE**: Passing in a manual config will override a set SENTRY_DSN Environment Variable
 *
 **/

import dotenv from 'dotenv';

import Logger from './logger';
import SentryClient from './sentry';
import type { LoggerConfig, LoggerInstance } from './logger';
import type { SentryConfig } from './sentry';

dotenv.config();

export type IntegrationClientConfig = {
    logger?: LoggerConfig;
    sentry?: SentryConfig;
};

class IntegrationClient {
    config?: IntegrationClientConfig;
    logger: LoggerInstance;
    sentry?: SentryClient;

    constructor(config?: IntegrationClientConfig) {
        this.config = config; // Store the passed in config

        this.logger = new Logger(config?.logger).init();

        if (config?.sentry?.dsn || process?.env?.SENTRY_DSN) {
            this.sentry = new SentryClient(
                config?.sentry || { ...config?.sentry, dsn: process?.env?.SENTRY_DSN || '' }
            ).init();
        }
    }
}

export default IntegrationClient;
