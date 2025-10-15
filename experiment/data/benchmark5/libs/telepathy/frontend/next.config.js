/** @type {import('next').NextConfig} */

// Lets us load the .env from the parent folder
const path = require("path");
const { parsed: localEnv } = require("dotenv").config({
  allowEmptyValues: false,
  path: path.resolve(__dirname, `../.env`),
});

const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  env: localEnv,
};

module.exports = nextConfig;
