import * as native from "../dist/index.js";

export const Config = native.Config;
export type Config = InstanceType<typeof native.Config>;

export type Limits = native.Limits;
export type Shutdown = native.Shutdown;
export type TlsConfig = native.TlsConfig;
export type MutualTls = native.MutualTls;
export type ConfigInit = native.ConfigInit;

export const DEBUG_PROFILE = native.DEBUG_PROFILE;
export const RELEASE_PROFILE = native.RELEASE_PROFILE;
export const DEFAULT_PROFILE = native.DEFAULT_PROFILE;

export const ADDRESS = native.ADDRESS;
export const PORT = native.PORT;
export const WORKERS = native.WORKERS;
export const MAX_BLOCKING = native.MAX_BLOCKING;
export const KEEP_ALIVE = native.KEEP_ALIVE;
export const IDENT = native.IDENT;
export const IP_HEADER = native.IP_HEADER;
export const LIMITS = native.LIMITS;
export const TLS = native.TLS;
export const SECRET_KEY = native.SECRET_KEY;
export const TEMP_DIR = native.TEMP_DIR;
export const LOG_LEVEL = native.LOG_LEVEL;
export const SHUTDOWN = native.SHUTDOWN;
export const CLI_COLORS = native.CLI_COLORS;
