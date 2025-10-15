import IntegrationClient from '../index';

describe('IntegrationClient', () => {
    test('Initializes with defaults', () => {
        const client = new IntegrationClient();

        expect(client.config).toBeUndefined();
        expect(client.logger).not.toBeUndefined();
    });

    describe('Sentry Integration', () => {
        test('Initializes when a DSN is present', () => {
            const client = new IntegrationClient({
                sentry: {
                    dsn: process?.env?.SENTRY_DSN || ''
                }
            });

            expect(client.sentry).not.toBeUndefined();
        });

        test('Not initialized without sentry config', () => {
            // Ensure the SENTRY_DSN is unset
            process.env.SENTRY_DSN = '';

            const client = new IntegrationClient();

            expect(client.config).toBeUndefined();
            expect(client.logger).not.toBeUndefined();
            expect(client.sentry).toBeUndefined();
        });
    });
});
