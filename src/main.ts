import {
  serve as nativeServe,
  file as nativeFile,
  Response as NativeResponse,
  Config as NativeConfig,
  Server,
  launch,
} from "../native/index.js";

import * as codegen from "./codegen.ts";

export interface ServeOptions {
  port?: number;
  hostname?: string;
  unix?: string;
  routes?: Record<string, any>;
  fetch?: (req: any) => any;
}

export function serve(options: ServeOptions): Server {
  return nativeServe(options);
}

export function file(path: string) {
  return nativeFile(path);
}

export class Response extends NativeResponse {
  static override json(
    data: any,
    init?: { status?: number; headers?: [string, string][] },
  ): Response {
    return NativeResponse.json(data, init);
  }

  static override redirect(url: string, status?: number): Response {
    return NativeResponse.redirect(url, status);
  }
}

export const Config = NativeConfig;

export const Alloy = {
  serve,
  file,
  Response,
  Config,
};

export { Server, launch, codegen };

export * from "./codegen.ts";

export function main(): string {
  return "Hello, world!";
}

export default Alloy;
