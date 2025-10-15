import winston from 'winston';

export function commonSetup() {
    const logger = winston.createLogger({
        level: 'info',
        format: winston.format.json(),
        defaultMeta: { service: 'user-service' },
        transports: [
            //
            // - Write all logs with importance level of `error` or less to `error.log`
            // - Write all logs with importance level of `info` or less to `combined.log`
            //
            // new winston.transports.File({ filename: 'error.log', level: 'error' }),
            // new winston.transports.File({ filename: 'combined.log' })
        ]
    });
    if (process.env.NODE_ENV !== 'production') {
        logger.add(
            new winston.transports.Console({
                format: winston.format.simple()
            })
        );
    }
    return {
        logger
    };
}

export function envSafeLoad(env_var: string) {
    const res = process.env[env_var];
    if (!res) {
        throw Error(`No ${env_var} in environment`);
    }
    return res;
}
