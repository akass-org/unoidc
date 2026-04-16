import { api } from "./client";

export interface PasskeyCredential {
  id: string;
  user_id: string;
  counter: number;
  device_type: string | null;
  backed_up: boolean | null;
  transports: string[] | null;
  display_name: string | null;
  created_at: string;
  last_used_at: string | null;
}

export const passkeyApi = {
  list: () => api.get("api/v1/passkey").json<PasskeyCredential[]>(),

  registerStart: () => api.post("api/v1/passkey/register/start").json<Record<string, unknown>>(),

  registerFinish: (data: Record<string, unknown>) =>
    api.post("api/v1/passkey/register/finish", { json: data }).json<{ success: boolean }>(),

  delete: (id: string) => api.delete(`api/v1/passkey/${id}`).json<{ success: boolean }>(),

  loginStart: () => api.post("api/v1/passkey/login/start").json<Record<string, unknown>>(),

  loginFinish: (data: Record<string, unknown>) =>
    api.post("api/v1/passkey/login/finish", { json: data }).json<{ success: boolean }>(),

  registerAnonStart: (data: { username: string; email: string; display_name: string }) =>
    api.post("api/v1/passkey/register-anon/start", { json: data }).json<{ options: Record<string, unknown>; temp_user_id: string }>(),

  registerAnonFinish: (data: Record<string, unknown>) =>
    api.post("api/v1/passkey/register-anon/finish", { json: data }).json<{ success: boolean }>(),
};
