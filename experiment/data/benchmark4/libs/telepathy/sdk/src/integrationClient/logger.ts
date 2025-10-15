/**
 * Logger object to be used in the integration client
 */

import winston from 'winston';
import type { LoggerOptions } from 'winston';

export type { LoggerOptions as LoggerConfig, Logger as LoggerInstance } from 'winston';

class Logger {
    logger?: winston.Logger;
    config?: LoggerOptions;
    defaultConfig: LoggerOptions;

    constructor(config?: LoggerOptions) {
        this.config = config; // store the passed in config

        this.defaultConfig = {
            level: 'info',
            format: winston.format.json(),
            defaultMeta: { service: 'user-service' },
            transports: []
        };
    }

    init() {
        this.logger = winston.createLogger({
            ...this.defaultConfig,
            ...this.config // override the defaults with whatever the user passed in
        });

        if (process.env.NODE_ENV !== 'production') {
            this.logger.add(
                new winston.transports.Console({
                    format: winston.format.simple()
                })
            );
        }

        return this.logger;
    }
}

export default Logger;
