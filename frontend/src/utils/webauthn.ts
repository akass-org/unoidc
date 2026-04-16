// WebAuthn 浏览器 API 工具函数

export function isWebAuthnSupported(): boolean {
  return typeof window !== "undefined" && !!window.PublicKeyCredential;
}

export function base64urlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

export function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary);
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

// 将服务端返回的 PublicKeyCredentialCreationOptions 转换为浏览器所需的格式
export function parseCreationOptions(options: unknown): CredentialCreationOptions {
  const opt = options as Record<string, unknown>;
  const publicKey = opt.publicKey as Record<string, unknown>;

  return {
    publicKey: {
      ...publicKey,
      challenge: base64urlToBuffer((publicKey.challenge as string) ?? ""),
      user: {
        ...(publicKey.user as Record<string, unknown>),
        id: base64urlToBuffer(((publicKey.user as Record<string, unknown>)?.id as string) ?? ""),
      },
      excludeCredentials: (
        (publicKey.excludeCredentials as Array<Record<string, unknown>>) ?? []
      ).map((c) => ({
        ...c,
        id: base64urlToBuffer((c.id as string) ?? ""),
      })),
    },
  } as CredentialCreationOptions;
}

// 将浏览器返回的 credential 转换为可发送给后端的 JSON
export async function serializeCredential(
  credential: PublicKeyCredential,
): Promise<Record<string, unknown>> {
  const response = credential.response as AuthenticatorAttestationResponse;

  const transports = (response.getTransports?.() ?? []) as string[];

  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      attestationObject: bufferToBase64url(response.attestationObject),
      transports,
    },
    clientExtensionResults: credential.getClientExtensionResults?.() ?? {},
  };
}

export function parseRequestOptions(options: unknown): CredentialRequestOptions {
  const opt = options as Record<string, unknown>;
  const publicKey = opt.publicKey as Record<string, unknown>;

  return {
    publicKey: {
      ...publicKey,
      challenge: base64urlToBuffer((publicKey.challenge as string) ?? ""),
      allowCredentials: (
        (publicKey.allowCredentials as Array<Record<string, unknown>>) ?? []
      ).map((c) => ({
        ...c,
        id: base64urlToBuffer((c.id as string) ?? ""),
      })),
    },
  } as CredentialRequestOptions;
}

export async function serializeAuthenticationCredential(
  credential: PublicKeyCredential,
): Promise<Record<string, unknown>> {
  const response = credential.response as AuthenticatorAssertionResponse;

  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      authenticatorData: bufferToBase64url(response.authenticatorData),
      signature: bufferToBase64url(response.signature),
      userHandle: response.userHandle
        ? bufferToBase64url(response.userHandle)
        : undefined,
    },
    clientExtensionResults: credential.getClientExtensionResults?.() ?? {},
  };
}

export async function startAuthentication(options: unknown): Promise<Record<string, unknown>> {
  const credential = (await navigator.credentials.get(
    parseRequestOptions(options),
  )) as PublicKeyCredential | null;

  if (!credential) {
    throw new Error("User cancelled passkey authentication");
  }

  return serializeAuthenticationCredential(credential);
}

export async function startRegistration(options: unknown): Promise<Record<string, unknown>> {
  const credential = (await navigator.credentials.create(
    parseCreationOptions(options),
  )) as PublicKeyCredential | null;

  if (!credential) {
    throw new Error("User cancelled passkey creation");
  }

  return serializeCredential(credential);
}
