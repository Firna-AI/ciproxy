import { getDefaultConfig } from "expo/metro-config.js";

const config = getDefaultConfig(import.meta.dirname);
config.resolver.assetExts.push("woff2");

export default config;
