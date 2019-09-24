import webcrypto from "@trust/webcrypto";
import { TextEncoder, TextDecoder } from "text-encoding";

window.crypto = webcrypto;
window.TextEncoder = TextEncoder;
window.TextDecoder = TextDecoder;
