import { useState, useEffect } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Shield, Eye, EyeOff, Fingerprint } from "lucide-react";
import { getErrorMessage } from "#src/api/client";
import { authApi } from "#src/api/auth";
import { passkeyApi } from "#src/api/passkey";
import { LoginPageWrapper } from "#src/components/LoginLayout";
import { ThemeToggle } from "#src/components/ThemeToggle";
import { useUIConfigStore } from "#src/stores/theme";
import { useSessionStore } from "#src/stores/session";
import { Input, Button, useToast } from "#src/components/ui";

export function RegisterPage() {
  const navigate = useNavigate();
  const { brandName } = useUIConfigStore();
  const { setUser } = useSessionStore();
  const { addToast } = useToast();

  const [config, setConfig] = useState({ enable_passkey_signup: false });
  const [regMode, setRegMode] = useState<"password" | "passkey">("password");

  const [formData, setFormData] = useState({
    username: "",
    email: "",
    displayName: "",
    password: "",
    confirmPassword: "",
  });
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const [passkeyForm, setPasskeyForm] = useState({
    username: "",
    email: "",
    displayName: "",
  });
  const [passkeyLoading, setPasskeyLoading] = useState(false);

  useEffect(() => {
    authApi
      .getPublicConfig()
      .then((data) => {
        setConfig({
          enable_passkey_signup: (data as { enable_passkey_signup?: boolean }).enable_passkey_signup ?? true,
        });
      })
      .catch(() => {
        setConfig({ enable_passkey_signup: true });
      });
  }, []);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (formData.password !== formData.confirmPassword) {
      setError("两次输入的密码不一致");
      return;
    }

    setLoading(true);
    try {
      // Register
      await authApi.register({
        username: formData.username,
        email: formData.email,
        display_name: formData.displayName,
        password: formData.password,
      });

      // Auto login
      await authApi.login(formData.username, formData.password);
      const session = (await authApi.getSession()) as {
        user: {
          id: string;
          username: string;
          email: string;
          display_name: string;
          picture?: string;
          is_admin: boolean;
        };
      };
      setUser(session.user);

      // Redirect to profile
      navigate("/profile");
    } catch (err: unknown) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function handlePasskeyRegister() {
    setError("");
    if (!window.PublicKeyCredential) {
      setError("您的浏览器不支持 passkey，请使用密码注册");
      return;
    }
    setPasskeyLoading(true);
    try {
      const startRes = (await passkeyApi.registerAnonStart({
        username: passkeyForm.username,
        email: passkeyForm.email,
        display_name: passkeyForm.displayName,
      })) as unknown as { options: PublicKeyCredentialCreationOptions };

      const credential = (await navigator.credentials.create({
        publicKey: startRes.options,
      })) as PublicKeyCredential | null;
      if (!credential) {
        setError("操作已取消");
        setPasskeyLoading(false);
        return;
      }

      const response = credential.response as AuthenticatorAttestationResponse;

      const finishPayload = {
        username: passkeyForm.username,
        email: passkeyForm.email,
        display_name: passkeyForm.displayName,
        id: credential.id,
        rawId: btoa(String.fromCharCode(...new Uint8Array(credential.rawId as ArrayBuffer)))
          .replace(/\+/g, "-")
          .replace(/\//g, "_")
          .replace(/=/g, ""),
        type: credential.type,
        response: {
          clientDataJSON: btoa(String.fromCharCode(...new Uint8Array(response.clientDataJSON)))
            .replace(/\+/g, "-")
            .replace(/\//g, "_")
            .replace(/=/g, ""),
          attestationObject: btoa(String.fromCharCode(...new Uint8Array(response.attestationObject)))
            .replace(/\+/g, "-")
            .replace(/\//g, "_")
            .replace(/=/g, ""),
          authenticatorData: (response as unknown as { getAuthenticatorData?: () => ArrayBuffer }).getAuthenticatorData
            ? btoa(
                String.fromCharCode(
                  ...new Uint8Array(
                    (response as unknown as { getAuthenticatorData: () => ArrayBuffer }).getAuthenticatorData(),
                  ),
                ),
              )
                .replace(/\+/g, "-")
                .replace(/\//g, "_")
                .replace(/=/g, "")
            : undefined,
          transports: (response as unknown as { transports?: string[] }).transports ?? [],
        },
        clientExtensionResults:
          (
            credential as unknown as { getClientExtensionResults?: () => Record<string, unknown> }
          ).getClientExtensionResults?.() || {},
      };

      await passkeyApi.registerAnonFinish(finishPayload);
      addToast({ type: "success", title: "注册成功", message: "请登录" });
      navigate("/login");
    } catch (err: unknown) {
      let message = "Passkey 注册失败，请重试";
      if (err instanceof Error && err.name === "NotAllowedError") {
        message = "操作已取消";
      }
      setError(message);
    } finally {
      setPasskeyLoading(false);
    }
  }

  return (
    <LoginPageWrapper>
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center justify-center w-8 h-8 rounded-md bg-black dark:bg-white">
            <Shield className="w-4 h-4 text-white dark:text-black" />
          </div>
          <span className="text-sm font-bold text-gray-900 dark:text-white">{brandName}</span>
        </div>
        <ThemeToggle />
      </div>

      {/* Title */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-1">创建账户</h1>
        <p className="text-sm text-gray-500 dark:text-gray-500">填写以下信息完成账户注册</p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-5 p-3 rounded-lg bg-red-500/[0.08] border border-red-500/[0.16]">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Mode Switch */}
      {config.enable_passkey_signup && (
        <div className="mb-5">
          <div className="inline-flex w-full p-1 bg-gray-100 dark:bg-white/[0.04] rounded-lg border border-gray-200 dark:border-white/[0.08]">
            <button
              type="button"
              onClick={() => setRegMode("password")}
              className={`flex-1 px-4 py-1.5 text-sm font-medium rounded-md transition-colors ${
                regMode === "password"
                  ? "bg-white dark:bg-white/[0.08] text-gray-900 dark:text-white shadow-sm"
                  : "text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
              }`}
            >
              密码注册
            </button>
            <button
              type="button"
              onClick={() => setRegMode("passkey")}
              className={`flex-1 px-4 py-1.5 text-sm font-medium rounded-md transition-colors ${
                regMode === "passkey"
                  ? "bg-white dark:bg-white/[0.08] text-gray-900 dark:text-white shadow-sm"
                  : "text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
              }`}
            >
              Passkey 注册
            </button>
          </div>
        </div>
      )}

      {/* Password Form */}
      {(!config.enable_passkey_signup || regMode === "password") && (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <Input
              label="用户名"
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
              placeholder="用户名"
              required
              autoFocus
            />

            <Input
              label="显示名称"
              value={formData.displayName}
              onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
              placeholder="显示名称"
              required
            />
          </div>

          <Input
            label="邮箱"
            type="email"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            placeholder="your@email.com"
            required
          />

          <div className="relative">
            <Input
              label="密码"
              type={showPassword ? "text" : "password"}
              value={formData.password}
              onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              placeholder="至少8位字符"
              required
              minLength={8}
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-[34px] text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
            >
              {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
          </div>

          <Input
            label="确认密码"
            type={showPassword ? "text" : "password"}
            value={formData.confirmPassword}
            onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
            placeholder="再次输入密码"
            required
          />

          <button
            type="submit"
            disabled={loading}
            style={{ backgroundColor: "#ffffff", color: "#000000" }}
            className="w-full py-3 px-4 font-bold text-sm rounded-md hover:bg-gray-100 btn-transition disabled:opacity-50 disabled:cursor-not-allowed border border-white"
          >
            {loading ? (
              <span className="flex items-center justify-center gap-2">
                <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                    fill="none"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                注册中...
              </span>
            ) : (
              "创建账户"
            )}
          </button>
        </form>
      )}

      {/* Passkey Form */}
      {config.enable_passkey_signup && regMode === "passkey" && (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <Input
              label="用户名"
              value={passkeyForm.username}
              onChange={(e) => setPasskeyForm({ ...passkeyForm, username: e.target.value })}
              placeholder="用户名"
              required
              autoFocus
            />
            <Input
              label="显示名称"
              value={passkeyForm.displayName}
              onChange={(e) => setPasskeyForm({ ...passkeyForm, displayName: e.target.value })}
              placeholder="显示名称"
              required
            />
          </div>
          <Input
            label="邮箱"
            type="email"
            value={passkeyForm.email}
            onChange={(e) => setPasskeyForm({ ...passkeyForm, email: e.target.value })}
            placeholder="your@email.com"
            required
          />
          <Button
            type="button"
            size="lg"
            className="w-full"
            loading={passkeyLoading}
            disabled={passkeyLoading}
            icon={<Fingerprint className="w-4 h-4" />}
            onClick={handlePasskeyRegister}
          >
            创建 Passkey
          </Button>
        </div>
      )}

      {/* Footer */}
      <div className="mt-6 pt-6 border-t border-gray-200 dark:border-white/[0.06] text-center">
        <p className="text-sm text-gray-500">
          已有账户？
          <Link
            to="/login"
            className="ml-1 text-gray-900 hover:underline dark:text-white transition-colors"
          >
            立即登录
          </Link>
        </p>
      </div>
    </LoginPageWrapper>
  );
}
