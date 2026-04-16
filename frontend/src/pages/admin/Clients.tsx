import { useState, useEffect } from "react";
import {
  Shield,
  Search,
  Plus,
  Key,
  Trash2,
  Edit,
  Copy,
  Check,
  Minus,
  ChevronDown,
  ChevronUp,
  Link as LinkIcon,
  Tags,
} from "lucide-react";
import { adminApi } from "#src/api/admin";
import { useApi } from "#src/hooks";
import { Card, Button, Input, Modal, Badge, EmptyState, useToast } from "#src/components/ui";
import { getErrorMessage } from "#src/api/client";

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`;
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`;

type OidcScope = "openid" | "email" | "profile" | "groups";

interface OidcPreviewUser {
  display_name: string;
  preferred_username: string;
  email: string;
  email_verified: boolean;
  given_name: string;
  family_name: string;
  picture: string;
  sub: string;
  groups: string[];
}

interface AdminUser {
  id: string;
  username: string;
  email: string;
  display_name: string;
  given_name?: string | null;
  family_name?: string | null;
  picture?: string | null;
  groups: string[];
  is_admin: boolean;
  is_active: boolean;
  created_at: string;
}

interface OidcPreviewPayload {
  idToken: Record<string, unknown>;
  accessToken: Record<string, unknown>;
  userinfo: Record<string, unknown>;
}

interface Group {
  id: string;
  name: string;
  description?: string | null;
  member_count: number;
  created_at: string;
}

interface Client {
  id: string;
  client_id: string;
  name: string;
  description?: string;
  redirect_uris: string[];
  post_logout_redirect_uris?: string[];
  allowed_group_ids?: string[];
  allowed_groups?: string[];
  is_active: boolean;
  enable_silent_authorize: boolean;
  created_at: string;
  last_used?: string;
}

const allScopes: OidcScope[] = ["openid", "email", "profile", "groups"];

function getUserInitials(user: AdminUser) {
  const base = user.display_name || user.username || "?";
  const parts = base.trim().split(/\s+/);
  return parts.length > 1
    ? `${parts[0][0] || ""}${parts[1][0] || ""}`.toUpperCase()
    : base.slice(0, 2).toUpperCase();
}

function toOidcPreviewUser(user: AdminUser): OidcPreviewUser {
  return {
    display_name: user.display_name,
    preferred_username: user.username,
    email: user.email,
    email_verified: user.is_active,
    given_name: user.given_name || "",
    family_name: user.family_name || "",
    picture: user.picture || "",
    sub: user.id,
    groups: user.groups,
  };
}

function formatJsonValue(value: unknown) {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "boolean") return value ? "true" : "false";
  if (typeof value === "number") return String(value);
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return `${value.length} 项`;
  return "对象";
}

function JsonTree({ value }: { value: unknown }) {
  if (Array.isArray(value)) {
    return (
      <div className="space-y-2">
        {value.length === 0 ? (
          <div className="text-xs text-gray-400 italic">[]</div>
        ) : (
          value.map((item, index) => (
            <div
              key={index}
              className="rounded-lg border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/20 px-3 py-2"
            >
              <div className="text-[10px] uppercase tracking-wider text-gray-400 mb-1">
                [{index}]
              </div>
              <JsonTree value={item} />
            </div>
          ))
        )}
      </div>
    );
  }

  if (value && typeof value === "object") {
    return (
      <div className="space-y-2">
        {Object.entries(value as Record<string, unknown>).map(([key, entryValue]) => {
          const hasNestedObject = entryValue !== null && typeof entryValue === "object";

          return (
            <div
              key={key}
              className="rounded-lg border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/20 px-3 py-2"
            >
              <div className="flex items-start justify-between gap-3">
                <span className="text-[10px] uppercase tracking-wider text-gray-400 shrink-0">
                  {key}
                </span>
                <span className="text-xs text-gray-700 dark:text-gray-300 text-right break-all">
                  {formatJsonValue(entryValue)}
                </span>
              </div>
              {hasNestedObject && (
                <div className="mt-2 pl-3 border-l border-gray-200 dark:border-white/[0.08]">
                  <JsonTree value={entryValue} />
                </div>
              )}
            </div>
          );
        })}
      </div>
    );
  }

  return (
    <span className="text-xs text-gray-700 dark:text-gray-300 break-all">
      {formatJsonValue(value)}
    </span>
  );
}

function buildOidcPreview(
  user: OidcPreviewUser,
  scopes: OidcScope[],
  clientId: string,
  issuer: string,
): OidcPreviewPayload {
  const now = new Date();
  const expiresAt = new Date(now.getTime() + 60 * 60 * 1000);

  const idTokenBase = {
    aud: [clientId],
    exp: expiresAt.toISOString(),
    iat: now.toISOString(),
    iss: issuer,
    jti: crypto.randomUUID(),
    sub: user.sub,
  };

  const accessTokenBase = {
    aud: [clientId],
    exp: expiresAt.toISOString(),
    iat: now.toISOString(),
    iss: issuer,
    jti: crypto.randomUUID(),
    sub: user.sub,
  };

  const has = (scope: OidcScope) => scopes.includes(scope);

  const idToken = {
    ...idTokenBase,
    type: "id-token",
    ...(has("profile")
      ? {
          display_name: user.display_name,
          given_name: user.given_name,
          family_name: user.family_name,
          name: user.display_name,
          preferred_username: user.preferred_username,
          picture: user.picture,
        }
      : {}),
    ...(has("email")
      ? {
          email: user.email,
          email_verified: user.email_verified,
        }
      : {}),
    ...(has("groups")
      ? {
          groups: user.groups,
        }
      : {}),
  };

  const accessToken = {
    ...accessTokenBase,
    type: "oauth-access-token",
  };

  const userinfo = {
    sub: user.sub,
    ...(has("profile")
      ? {
          display_name: user.display_name,
          given_name: user.given_name,
          family_name: user.family_name,
          name: user.display_name,
          preferred_username: user.preferred_username,
          picture: user.picture,
        }
      : {}),
    ...(has("email")
      ? {
          email: user.email,
          email_verified: user.email_verified,
        }
      : {}),
    ...(has("groups")
      ? {
          groups: user.groups,
        }
      : {}),
  };

  return { idToken, accessToken, userinfo };
}

export function AdminClients() {
  const [clients, setClients] = useState<Client[]>([]);
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [groups, setGroups] = useState<Group[]>([]);
  const [filteredClients, setFilteredClients] = useState<Client[]>([]);
  const [search, setSearch] = useState("");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingClient, setEditingClient] = useState<Client | null>(null);
  const [deletingClient, setDeletingClient] = useState<Client | null>(null);
  const [resettingClient, setResettingClient] = useState<Client | null>(null);
  const [newClientSecret, setNewClientSecret] = useState<{ client: Client; secret: string } | null>(
    null,
  );
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [selectedUserId, setSelectedUserId] = useState("");
  const [selectedScopes, setSelectedScopes] = useState<OidcScope[]>([
    "openid",
    "email",
    "profile",
    "groups",
  ]);
  const [showOidcPreview, setShowOidcPreview] = useState(false);
  const [selectedPreviewPanel, setSelectedPreviewPanel] = useState<
    "id-token" | "access-token" | "userinfo"
  >("id-token");
  const { addToast } = useToast();

  // Form states
  const [formData, setFormData] = useState({
    name: "",
    description: "",
    redirect_uris: [""],
    post_logout_redirect_uris: [""],
    allowed_group_ids: [] as string[],
    enable_silent_authorize: false,
  });

  // Load clients
  useEffect(() => {
    loadClients();
  }, []);

  // Load users
  useEffect(() => {
    loadUsers();
  }, []);

  // Load groups
  useEffect(() => {
    loadGroups();
  }, []);

  // Filter clients
  useEffect(() => {
    const filtered = clients.filter(
      (c) =>
        c.name.toLowerCase().includes(search.toLowerCase()) ||
        c.client_id.toLowerCase().includes(search.toLowerCase()),
    );
    setFilteredClients(filtered);
  }, [clients, search]);

  const loadClients = async () => {
    try {
      const data = (await adminApi.getClients()) as Client[];
      setClients(data);
    } catch (err) {
      addToast({
        type: "error",
        title: "加载失败",
        message: getErrorMessage(err),
      });
    }
  };

  const loadUsers = async () => {
    try {
      const data = (await adminApi.getUsers()) as AdminUser[];
      setUsers(data);
    } catch (err) {
      addToast({
        type: "error",
        title: "加载用户失败",
        message: getErrorMessage(err),
      });
    }
  };

  const loadGroups = async () => {
    try {
      const data = (await adminApi.getGroups()) as Group[];
      setGroups(data);
    } catch (err) {
      addToast({
        type: "error",
        title: "加载用户组失败",
        message: getErrorMessage(err),
      });
    }
  };

  const { loading: creating, execute: createClient } = useApi(adminApi.createClient, {
    successMessage: "应用创建成功",
    onSuccess: (data) => {
      const result = data as { client: Client; client_secret: string };
      setNewClientSecret({ client: result.client, secret: result.client_secret });
      setShowCreateModal(false);
      setFormData({
        name: "",
        description: "",
        redirect_uris: [""],
        post_logout_redirect_uris: [""],
        allowed_group_ids: [],
        enable_silent_authorize: false,
      });
      loadClients();
    },
  });

  const { loading: updating, execute: updateClient } = useApi(
    (id: string, data: Record<string, unknown>) => adminApi.updateClient(id, data),
    {
      successMessage: "应用更新成功",
      onSuccess: () => {
        setEditingClient(null);
        loadClients();
      },
    },
  );

  const { loading: deleting, execute: deleteClient } = useApi(
    (id: string) => adminApi.deleteClient(id),
    {
      successMessage: "应用已删除",
      onSuccess: () => {
        setDeletingClient(null);
        loadClients();
      },
    },
  );

  const { loading: resetting, execute: resetSecret } = useApi(
    (id: string) => adminApi.resetClientSecret(id),
    {
      successMessage: "密钥已重置",
      onSuccess: (data) => {
        const result = data as { client: Client; client_secret: string };
        setNewClientSecret({ client: result.client, secret: result.client_secret });
        setResettingClient(null);
        loadClients();
      },
    },
  );

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    await createClient({
      name: formData.name,
      description: formData.description,
      redirect_uris: formData.redirect_uris.filter(Boolean),
      post_logout_redirect_uris:
        formData.post_logout_redirect_uris.filter(Boolean).length > 0
          ? formData.post_logout_redirect_uris.filter(Boolean)
          : undefined,
      allowed_group_ids: formData.allowed_group_ids,
      enable_silent_authorize: formData.enable_silent_authorize,
    });
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingClient) return;
    await updateClient(editingClient.id, {
      name: editingClient.name,
      description: editingClient.description,
      redirect_uris: editingClient.redirect_uris,
      post_logout_redirect_uris: editingClient.post_logout_redirect_uris,
      is_active: editingClient.is_active,
      allowed_group_ids: editingClient.allowed_group_ids ?? [],
      enable_silent_authorize: editingClient.enable_silent_authorize,
    });
  };

  const handleDelete = async () => {
    if (!deletingClient) return;
    await deleteClient(deletingClient.id);
  };

  const handleResetSecret = async () => {
    if (!resettingClient) return;
    await resetSecret(resettingClient.id);
  };

  const handleCopy = async (text: string, field: string) => {
    await navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  const getOidcEndpoints = () => {
    const baseUrl = window.location.origin;
    return {
      issuer: baseUrl,
      well_known_endpoint: `${baseUrl}/.well-known/openid-configuration`,
      authorization_endpoint: `${baseUrl}/authorize`,
      token_endpoint: `${baseUrl}/token`,
      userinfo_endpoint: `${baseUrl}/userinfo`,
      jwks_uri: `${baseUrl}/jwks.json`,
      end_session_endpoint: `${baseUrl}/logout`,
    };
  };

  const getEligibleUsers = (client: Client) => {
    const activeUsers = users.filter((user) => user.is_active);
    if (!client.allowed_groups || client.allowed_groups.length === 0) {
      return activeUsers;
    }

    return activeUsers.filter((user) =>
      user.groups.some((group) => client.allowed_groups?.includes(group)),
    );
  };

  const toggleScope = (scope: OidcScope) => {
    if (scope === "openid") return;

    setSelectedScopes((current) => {
      if (current.includes(scope)) {
        return current.filter((item) => item !== scope);
      }

      return [...current, scope];
    });
  };

  const renderUserChip = (user: AdminUser, active: boolean, onClick: () => void) => (
    <button
      key={user.id}
      type="button"
      onClick={onClick}
      className={`flex min-w-[176px] items-center gap-3 rounded-xl border px-3 py-2 text-left transition-colors ${active ? "border-black bg-black text-white dark:border-white dark:bg-white dark:text-black" : "border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50 dark:border-white/[0.08] dark:bg-black/20 dark:text-gray-300 dark:hover:border-white/[0.16] dark:hover:bg-white/[0.04]"}`}
    >
      <div
        className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-xs font-semibold ${active ? "bg-white/15 text-white dark:bg-black/10 dark:text-black" : "bg-gray-100 text-gray-600 dark:bg-white/[0.06] dark:text-gray-200"}`}
      >
        {getUserInitials(user)}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="truncate text-sm font-medium">{user.display_name || user.username}</span>
          {user.is_admin && <Badge variant={active ? "default" : "warning"}>管理员</Badge>}
        </div>
        <p
          className={`truncate text-[11px] ${active ? "text-white/70 dark:text-black/70" : "text-gray-500 dark:text-gray-500"}`}
        >
          @{user.username}
        </p>
      </div>
    </button>
  );

  const handleCopyJson = async (payload: Record<string, unknown>, field: string) => {
    await handleCopy(JSON.stringify(payload, null, 2), field);
  };

  const toggleCreateGroup = (groupId: string) => {
    setFormData((current) => ({
      ...current,
      allowed_group_ids: current.allowed_group_ids.includes(groupId)
        ? current.allowed_group_ids.filter((id) => id !== groupId)
        : [...current.allowed_group_ids, groupId],
    }));
  };

  const toggleEditGroup = (groupId: string) => {
    if (!editingClient) return;

    setEditingClient({
      ...editingClient,
      allowed_group_ids: (editingClient.allowed_group_ids ?? []).includes(groupId)
        ? (editingClient.allowed_group_ids ?? []).filter((id) => id !== groupId)
        : [...(editingClient.allowed_group_ids ?? []), groupId],
    });
  };

  const renderGroupSelector = (
    selectedGroupIds: string[],
    onToggleGroup: (groupId: string) => void,
    emptyText: string,
  ) => (
    <div>
      <div className="flex items-center justify-between gap-3 mb-1.5">
        <label className="block text-sm font-medium text-gray-600 dark:text-gray-400">用户组</label>
        <span className="text-xs text-gray-500 dark:text-gray-600">
          {selectedGroupIds.length} 个已选
        </span>
      </div>
      {groups.length === 0 ? (
        <div className="rounded-lg border border-dashed border-gray-300 dark:border-white/[0.12] px-3 py-3 text-sm text-gray-500 dark:text-gray-600">
          {emptyText}
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-60 overflow-y-auto pr-1">
          {groups.map((group) => {
            const active = selectedGroupIds.includes(group.id);
            return (
              <button
                key={group.id}
                type="button"
                onClick={() => onToggleGroup(group.id)}
                className={`flex items-start gap-3 rounded-xl border px-3 py-2.5 text-left transition-colors ${active ? "border-black bg-black text-white dark:border-white dark:bg-white dark:text-black" : "border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50 dark:border-white/[0.08] dark:bg-black/20 dark:text-gray-300 dark:hover:border-white/[0.16] dark:hover:bg-white/[0.04]"}`}
              >
                <div
                  className={`mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-lg ${active ? "bg-white/15 text-white dark:bg-black/10 dark:text-black" : "bg-gray-100 text-gray-500 dark:bg-white/[0.06] dark:text-gray-300"}`}
                >
                  <Tags className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="truncate text-sm font-medium">{group.name}</span>
                    {active && (
                      <span className="rounded-full bg-white/15 px-2 py-0.5 text-[10px] uppercase tracking-wider dark:bg-black/10">
                        已选
                      </span>
                    )}
                  </div>
                  <p
                    className={`mt-0.5 truncate text-xs ${active ? "text-white/70 dark:text-black/70" : "text-gray-500 dark:text-gray-500"}`}
                  >
                    {group.description || "无描述"}
                  </p>
                  <p
                    className={`mt-1 text-[11px] ${active ? "text-white/60 dark:text-black/60" : "text-gray-400 dark:text-gray-500"}`}
                  >
                    {group.member_count} 位成员
                  </p>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );

  const renderPreviewPanel = (
    title: string,
    payload: Record<string, unknown>,
    fieldKey: string,
    note: string,
  ) => (
    <div className="rounded-xl border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/20 p-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between mb-3">
        <div>
          <h4 className="text-sm font-medium text-gray-900 dark:text-white">{title}</h4>
          <p className="text-xs text-gray-500 dark:text-gray-600 mt-1">{note}</p>
        </div>
        <button
          type="button"
          onClick={() => handleCopyJson(payload, fieldKey)}
          className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-white/[0.08] px-2.5 py-1.5 text-xs text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-white/[0.04] transition-colors"
        >
          {copiedField === fieldKey ? (
            <Check className="w-3.5 h-3.5 text-emerald-500" />
          ) : (
            <Copy className="w-3.5 h-3.5" />
          )}
          复制 JSON
        </button>
      </div>
      <div className="max-h-[340px] overflow-auto pr-1">
        <JsonTree value={payload} />
      </div>
    </div>
  );

  return (
    <div className="space-y-5" style={{ animation: "slideUp 0.3s ease-out" }}>
      <style>
        {fadeIn}
        {slideUp}
      </style>

      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">应用管理</h1>
          <p className="text-sm text-gray-500 mt-1">管理 OIDC 客户端和授权配置</p>
        </div>
        <Button
          onClick={() => {
            setFormData({
              name: "",
              description: "",
              redirect_uris: [""],
              post_logout_redirect_uris: [""],
              allowed_group_ids: [],
              enable_silent_authorize: false,
            });
            setShowCreateModal(true);
          }}
          size="sm"
        >
          <Plus className="w-4 h-4 mr-1.5" />
          创建应用
        </Button>
      </div>

      {/* Search */}
      <Card padding="sm">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="搜索应用..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent pl-9 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 focus:outline-none"
          />
        </div>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-3">
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-900 dark:text-white">{clients.length}</p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">应用</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-emerald-500">
            {clients.filter((c) => c.is_active).length}
          </p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">活跃</p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-600 dark:text-gray-300">
            {clients.filter((c) => c.last_used).length}
          </p>
          <p className="text-[11px] text-gray-500 uppercase tracking-wider mt-1">已使用</p>
        </Card>
      </div>

      {/* Clients List */}
      {filteredClients.length === 0 ? (
        <Card padding="lg">
          <EmptyState
            icon={<Shield className="w-8 h-8" />}
            title={search ? "无匹配结果" : "暂无应用"}
            description={search ? "尝试其他搜索词" : "点击上方按钮创建第一个应用"}
          />
        </Card>
      ) : (
        <div className="space-y-3">
          {filteredClients.map((client) => {
            const isExpanded = expandedId === client.id;

            return (
              <Card key={client.id} className="overflow-hidden group">
                {/* Header Row */}
                <div
                  className="flex items-start gap-4 px-5 py-4 cursor-pointer hover:bg-gray-50/60 dark:hover:bg-white/[0.02] transition-colors"
                  onClick={() => setExpandedId(isExpanded ? null : client.id)}
                >
                  {/* App Icon */}
                  <div className="w-11 h-11 rounded-xl bg-gradient-to-br from-gray-100 to-gray-50 dark:from-white/[0.06] dark:to-white/[0.02] border border-gray-200/80 dark:border-white/[0.06] flex items-center justify-center shrink-0 shadow-sm">
                    <Shield className="w-5 h-5 text-gray-600 dark:text-gray-400" />
                  </div>

                  {/* Main Content */}
                  <div className="flex-1 min-w-0 pt-0.5">
                    {/* Title Row */}
                    <div className="flex items-center gap-2.5 mb-2">
                      <h3 className="text-base font-semibold text-gray-900 dark:text-white leading-none">
                        {client.name}
                      </h3>
                      <Badge variant={client.is_active ? "success" : "error"} size="sm">
                        {client.is_active ? "活跃" : "禁用"}
                      </Badge>
                    </div>

                    {/* Client ID Row */}
                    <div className="flex items-center gap-2 mb-3">
                      <code className="text-xs font-mono text-gray-500 dark:text-gray-400 bg-gray-100/80 dark:bg-white/[0.04] px-2 py-1 rounded-md">
                        {client.client_id}
                      </code>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleCopy(client.client_id, `${client.id}-client-id`);
                        }}
                        className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded transition-colors"
                        title="复制 Client ID"
                      >
                        {copiedField === `${client.id}-client-id` ? (
                          <Check className="w-3.5 h-3.5 text-emerald-500" />
                        ) : (
                          <Copy className="w-3.5 h-3.5" />
                        )}
                      </button>
                    </div>

                    {/* Meta Info */}
                    <div className="flex flex-wrap items-center gap-3 text-xs text-gray-500">
                      <span className="inline-flex items-center gap-1.5">
                        <LinkIcon className="w-3.5 h-3.5 text-gray-400" />
                        {client.redirect_uris.length} 个回调
                      </span>
                      {client.allowed_groups && client.allowed_groups.length > 0 && (
                        <span className="text-gray-400">·</span>
                      )}
                      {client.allowed_groups && client.allowed_groups.length > 0 && (
                        <span>{client.allowed_groups.length} 个用户组</span>
                      )}
                      <span className="text-gray-400">·</span>
                      <span
                        className={
                          client.enable_silent_authorize ? "text-emerald-500" : "text-gray-400"
                        }
                      >
                        {client.enable_silent_authorize ? "无感登录开启" : "无感登录关闭"}
                      </span>
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-0.5 shrink-0 pt-0.5">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setEditingClient(client);
                      }}
                      className="p-2 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded-lg transition-colors"
                      title="编辑"
                    >
                      <Edit className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setResettingClient(client);
                      }}
                      className="p-2 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded-lg transition-colors"
                      title="重置密钥"
                    >
                      <Key className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeletingClient(client);
                      }}
                      className="p-2 text-gray-400 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg transition-colors"
                      title="删除"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                    <div className="ml-1 p-2 text-gray-400">
                      {isExpanded ? (
                        <ChevronUp className="w-4 h-4" />
                      ) : (
                        <ChevronDown className="w-4 h-4" />
                      )}
                    </div>
                  </div>
                </div>

                {/* Expanded Details */}
                {isExpanded && (
                  <div className="border-t border-gray-200 dark:border-white/[0.04] px-4 py-5 bg-gray-50/30 dark:bg-white/[0.02]">
                    {/* OIDC Config */}
                    <div className="space-y-3">
                      <div className="flex items-center justify-between gap-3">
                        <h4 className="text-xs font-medium text-gray-500 flex items-center gap-1.5">
                          <Shield className="w-3.5 h-3.5" />
                          OIDC 配置
                        </h4>
                        <span className="text-[10px] text-gray-500 uppercase tracking-wider">
                          基础端点
                        </span>
                      </div>
                      <div className="grid grid-cols-1 xl:grid-cols-2 gap-2.5">
                        {(() => {
                          const endpoints = getOidcEndpoints();
                          const items = [
                            { label: "Issuer", value: endpoints.issuer },
                            { label: "Well-Known", value: endpoints.well_known_endpoint },
                            { label: "Client ID", value: client.client_id },
                            { label: "授权端点", value: endpoints.authorization_endpoint },
                            { label: "令牌端点", value: endpoints.token_endpoint },
                            { label: "用户信息", value: endpoints.userinfo_endpoint },
                            { label: "JWKS", value: endpoints.jwks_uri },
                          ];
                          return items.map(({ label, value }) => (
                            <div
                              key={label}
                              className="flex items-center gap-2 group min-w-0 rounded-lg border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/20 px-3 py-2"
                            >
                              <span className="text-xs text-gray-500 w-20 shrink-0">{label}</span>
                              <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 truncate">
                                {value}
                              </code>
                              <button
                                onClick={() => handleCopy(value, `${client.id}-${label}`)}
                                className="p-1.5 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                              >
                                {copiedField === `${client.id}-${label}` ? (
                                  <Check className="w-3.5 h-3.5 text-emerald-500" />
                                ) : (
                                  <Copy className="w-3.5 h-3.5" />
                                )}
                              </button>
                            </div>
                          ));
                        })()}
                      </div>
                    </div>

                    {/* OIDC Preview */}
                    {(() => {
                      const eligibleUsers = getEligibleUsers(client);
                      const previewUser =
                        eligibleUsers.find((user) => user.id === selectedUserId) ??
                        eligibleUsers[0];
                      const previewData = previewUser
                        ? buildOidcPreview(
                            toOidcPreviewUser(previewUser),
                            selectedScopes,
                            client.client_id,
                            getOidcEndpoints().issuer,
                          )
                        : null;

                      return (
                        <div className="mt-5 rounded-2xl border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/20 p-4 lg:p-5">
                          <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                            <div className="min-w-0">
                              <h4 className="text-xs font-medium text-gray-500 mb-1 flex items-center gap-1.5">
                                <Shield className="w-3.5 h-3.5" />
                                OIDC 数据预览
                              </h4>
                              <p className="text-xs text-gray-500 dark:text-gray-600">
                                这部分是次级调试项，默认收起；点开后可切换用户和 scope
                                组合，单独查看一种令牌或 Userinfo。
                              </p>
                            </div>
                            <button
                              type="button"
                              onClick={() => setShowOidcPreview((current) => !current)}
                              className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-white/[0.08] px-3 py-1.5 text-xs text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-white/[0.04] transition-colors"
                            >
                              {showOidcPreview ? "收起预览" : "展开预览"}
                            </button>
                          </div>

                          {showOidcPreview && (
                            <div className="mt-4 space-y-4">
                              <div className="flex flex-wrap gap-2">
                                <button
                                  type="button"
                                  onClick={() => setSelectedScopes(["openid"])}
                                  className="rounded-full border border-gray-200 dark:border-white/[0.08] px-3 py-1.5 text-xs text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-white/[0.04] transition-colors"
                                >
                                  最小 scope
                                </button>
                                <button
                                  type="button"
                                  onClick={() => setSelectedScopes(allScopes)}
                                  className="rounded-full border border-gray-200 dark:border-white/[0.08] px-3 py-1.5 text-xs text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-gray-50 dark:hover:bg-white/[0.04] transition-colors"
                                >
                                  全选
                                </button>
                              </div>

                              <div className="space-y-2">
                                <div className="flex items-center justify-between gap-3">
                                  <span className="text-[10px] uppercase tracking-wider text-gray-400">
                                    选择用户
                                  </span>
                                  <span className="text-[10px] uppercase tracking-wider text-gray-400">
                                    {eligibleUsers.length} 位可访问用户
                                  </span>
                                </div>
                                <div className="flex gap-2 overflow-x-auto pb-1">
                                  {eligibleUsers.map((user) =>
                                    renderUserChip(user, selectedUserId === user.id, () =>
                                      setSelectedUserId(user.id),
                                    ),
                                  )}
                                  {eligibleUsers.length === 0 && (
                                    <div className="rounded-xl border border-dashed border-gray-200 dark:border-white/[0.08] px-3 py-2 text-xs text-gray-500 dark:text-gray-600">
                                      暂无可访问用户
                                    </div>
                                  )}
                                </div>
                              </div>

                              <div className="space-y-2">
                                <div className="flex items-center justify-between gap-3 mb-2">
                                  <span className="text-[10px] uppercase tracking-wider text-gray-400">
                                    选择 scope
                                  </span>
                                  <span className="text-[10px] text-gray-500 dark:text-gray-600">
                                    openid 必选
                                  </span>
                                </div>
                                <div className="flex flex-wrap gap-2">
                                  {allScopes.map((scope) => {
                                    const active =
                                      scope === "openid" || selectedScopes.includes(scope);
                                    return (
                                      <button
                                        key={scope}
                                        type="button"
                                        onClick={() => toggleScope(scope)}
                                        className={`rounded-full px-3 py-1.5 text-xs transition-colors border ${active ? "border-emerald-500 bg-emerald-500/[0.12] text-emerald-400" : "border-gray-200 text-gray-500 hover:text-gray-900 dark:border-white/[0.08] dark:text-gray-400 dark:hover:text-white"}`}
                                      >
                                        {scope}
                                      </button>
                                    );
                                  })}
                                </div>
                              </div>

                              {previewData ? (
                                <div className="rounded-xl border border-gray-200 dark:border-white/[0.06] bg-gray-50 dark:bg-black/20 p-3">
                                  <div className="flex flex-wrap gap-2 mb-3">
                                    {(
                                      [
                                        { id: "id-token", label: "ID Token" },
                                        { id: "access-token", label: "Access Token" },
                                        { id: "userinfo", label: "Userinfo" },
                                      ] as const
                                    ).map((item) => (
                                      <button
                                        key={item.id}
                                        type="button"
                                        onClick={() => setSelectedPreviewPanel(item.id)}
                                        className={`rounded-full px-3 py-1.5 text-xs transition-colors border ${selectedPreviewPanel === item.id ? "border-black bg-black text-white dark:border-white dark:bg-white dark:text-black" : "border-gray-200 text-gray-500 hover:text-gray-900 dark:border-white/[0.08] dark:text-gray-400 dark:hover:text-white"}`}
                                      >
                                        {item.label}
                                      </button>
                                    ))}
                                  </div>

                                  {selectedPreviewPanel === "id-token" &&
                                    renderPreviewPanel(
                                      "ID Token",
                                      previewData.idToken,
                                      `${client.id}-id-token-preview`,
                                      "身份令牌，按当前 scope 动态展示声明",
                                    )}
                                  {selectedPreviewPanel === "access-token" &&
                                    renderPreviewPanel(
                                      "Access Token",
                                      previewData.accessToken,
                                      `${client.id}-access-token-preview`,
                                      "访问令牌，仅展示令牌元数据",
                                    )}
                                  {selectedPreviewPanel === "userinfo" &&
                                    renderPreviewPanel(
                                      "Userinfo",
                                      previewData.userinfo,
                                      `${client.id}-userinfo-preview`,
                                      "Userinfo 响应，和 scope 绑定",
                                    )}
                                </div>
                              ) : (
                                <div className="rounded-xl border border-dashed border-gray-200 dark:border-white/[0.08] px-4 py-6 text-sm text-gray-500 dark:text-gray-400">
                                  当前没有可用于预览的注册用户。
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      );
                    })()}

                    {/* Callback URLs */}
                    <div className="mt-4">
                      <h4 className="text-xs font-medium text-gray-500 mb-2 flex items-center gap-1.5">
                        <LinkIcon className="w-3.5 h-3.5" />
                        回调地址
                      </h4>
                      <div className="space-y-2">
                        <div>
                          <span className="text-[10px] text-gray-400 uppercase tracking-wider">
                            登录回调
                          </span>
                          <div className="mt-1 space-y-1.5">
                            {client.redirect_uris.length > 0 ? (
                              client.redirect_uris.map((uri, i) => (
                                <div key={i} className="flex items-center gap-2 group">
                                  <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-black/20 px-3 py-2 rounded border border-gray-200 dark:border-white/[0.06] truncate">
                                    {uri}
                                  </code>
                                  <button
                                    onClick={() => handleCopy(uri, `redirect-${i}`)}
                                    className="p-1.5 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                                  >
                                    {copiedField === `redirect-${i}` ? (
                                      <Check className="w-3.5 h-3.5" />
                                    ) : (
                                      <Copy className="w-3.5 h-3.5" />
                                    )}
                                  </button>
                                </div>
                              ))
                            ) : (
                              <div className="flex items-center gap-2">
                                <span className="flex-1 text-xs text-gray-400 italic bg-white dark:bg-black/20 px-3 py-2 rounded border border-dashed border-gray-200 dark:border-white/[0.06]">
                                  未配置
                                </span>
                                <div className="w-8 shrink-0"></div>
                              </div>
                            )}
                          </div>
                        </div>
                        {client.post_logout_redirect_uris &&
                          client.post_logout_redirect_uris.length > 0 && (
                            <div>
                              <span className="text-[10px] text-gray-400 uppercase tracking-wider">
                                登出回调
                              </span>
                              <div className="mt-1 space-y-1.5">
                                {client.post_logout_redirect_uris.map((uri, i) => (
                                  <div key={i} className="flex items-center gap-2 group">
                                    <code className="flex-1 text-xs font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-black/20 px-3 py-2 rounded border border-gray-200 dark:border-white/[0.06] truncate">
                                      {uri}
                                    </code>
                                    <button
                                      onClick={() => handleCopy(uri, `logout-${i}`)}
                                      className="p-1.5 text-gray-400 hover:text-gray-700 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                                    >
                                      {copiedField === `logout-${i}` ? (
                                        <Check className="w-3.5 h-3.5" />
                                      ) : (
                                        <Copy className="w-3.5 h-3.5" />
                                      )}
                                    </button>
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}
                      </div>
                    </div>
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}

      {/* Create Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => {
          setShowCreateModal(false);
          setFormData({
            name: "",
            description: "",
            redirect_uris: [""],
            post_logout_redirect_uris: [""],
            allowed_group_ids: [],
            enable_silent_authorize: false,
          });
        }}
        title="创建应用"
        description="注册新的 OIDC 客户端"
        footer={
          <>
            <Button variant="ghost" onClick={() => setShowCreateModal(false)}>
              取消
            </Button>
            <Button onClick={handleCreate} loading={creating}>
              创建
            </Button>
          </>
        }
      >
        <form className="space-y-4">
          <Input
            label="应用名称 *"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="例如: 我的应用"
            required
          />
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
              描述
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="应用描述（可选）"
              rows={2}
              className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all resize-none"
            />
          </div>

          {/* Login Redirect URIs */}
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
              登录回调 URI
            </label>
            <div className="space-y-2">
              {formData.redirect_uris.map((uri, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="url"
                    value={uri}
                    onChange={(e) => {
                      const newUris = [...formData.redirect_uris];
                      newUris[index] = e.target.value;
                      setFormData({ ...formData, redirect_uris: newUris });
                    }}
                    placeholder="https://example.com/callback"
                    className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                  />
                  <button
                    type="button"
                    onClick={() =>
                      setFormData({
                        ...formData,
                        redirect_uris: formData.redirect_uris.filter((_, i) => i !== index),
                      })
                    }
                    className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                  >
                    <Minus className="w-4 h-4" />
                  </button>
                </div>
              ))}
              <button
                type="button"
                onClick={() =>
                  setFormData({ ...formData, redirect_uris: [...formData.redirect_uris, ""] })
                }
                className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                添加回调地址
              </button>
            </div>
          </div>

          {/* Logout Redirect URIs */}
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
              登出回调 URI（可选）
            </label>
            <div className="space-y-2">
              {formData.post_logout_redirect_uris.map((uri, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="url"
                    value={uri}
                    onChange={(e) => {
                      const newUris = [...formData.post_logout_redirect_uris];
                      newUris[index] = e.target.value;
                      setFormData({ ...formData, post_logout_redirect_uris: newUris });
                    }}
                    placeholder="https://example.com/logout"
                    className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                  />
                  <button
                    type="button"
                    onClick={() =>
                      setFormData({
                        ...formData,
                        post_logout_redirect_uris: formData.post_logout_redirect_uris.filter(
                          (_, i) => i !== index,
                        ),
                      })
                    }
                    className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                  >
                    <Minus className="w-4 h-4" />
                  </button>
                </div>
              ))}
              <button
                type="button"
                onClick={() =>
                  setFormData({
                    ...formData,
                    post_logout_redirect_uris: [...formData.post_logout_redirect_uris, ""],
                  })
                }
                className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                添加登出回调
              </button>
            </div>
          </div>

          {renderGroupSelector(
            formData.allowed_group_ids,
            toggleCreateGroup,
            "暂无可选用户组，请先创建用户组",
          )}

          <label className="flex items-start gap-2 cursor-pointer rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-50 dark:bg-white/[0.03] px-3 py-3">
            <input
              type="checkbox"
              checked={formData.enable_silent_authorize}
              onChange={(e) =>
                setFormData({ ...formData, enable_silent_authorize: e.target.checked })
              }
              className="mt-0.5 w-4 h-4 rounded border-gray-200 dark:border-white/[0.12] bg-gray-50 dark:bg-white/[0.04] text-white focus:ring-white/20"
            />
            <span>
              <span className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                启用无感授权
              </span>
              <span className="block text-xs text-gray-500 dark:text-gray-500 mt-0.5">
                已登录且已同意过的用户，可直接返回 code，不再显示授权页
              </span>
            </span>
          </label>
        </form>
      </Modal>

      {/* Edit Modal */}
      {editingClient && (
        <Modal
          isOpen={!!editingClient}
          onClose={() => setEditingClient(null)}
          title="编辑应用"
          footer={
            <>
              <Button variant="ghost" onClick={() => setEditingClient(null)}>
                取消
              </Button>
              <Button onClick={handleUpdate} loading={updating}>
                保存
              </Button>
            </>
          }
        >
          <form className="space-y-4">
            <Input
              label="应用名称"
              value={editingClient.name}
              onChange={(e) => setEditingClient({ ...editingClient, name: e.target.value })}
            />
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                描述
              </label>
              <textarea
                value={editingClient.description || ""}
                onChange={(e) =>
                  setEditingClient({ ...editingClient, description: e.target.value })
                }
                rows={2}
                className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all resize-none"
              />
            </div>

            {renderGroupSelector(
              editingClient.allowed_group_ids ?? [],
              toggleEditGroup,
              "暂无可选用户组，请先创建用户组",
            )}

            {/* Login Redirect URIs */}
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                登录回调 URI
              </label>
              <div className="space-y-2">
                {editingClient.redirect_uris.map((uri, index) => (
                  <div key={index} className="flex gap-2">
                    <input
                      type="url"
                      value={uri}
                      onChange={(e) => {
                        const newUris = [...editingClient.redirect_uris];
                        newUris[index] = e.target.value;
                        setEditingClient({ ...editingClient, redirect_uris: newUris });
                      }}
                      placeholder="https://example.com/callback"
                      className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                    />
                    <button
                      type="button"
                      onClick={() =>
                        setEditingClient({
                          ...editingClient,
                          redirect_uris: editingClient.redirect_uris.filter((_, i) => i !== index),
                        })
                      }
                      className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                    >
                      <Minus className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  onClick={() =>
                    setEditingClient({
                      ...editingClient,
                      redirect_uris: [...editingClient.redirect_uris, ""],
                    })
                  }
                  className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  添加回调地址
                </button>
              </div>
            </div>

            {/* Logout Redirect URIs */}
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                登出回调 URI（可选）
              </label>
              <div className="space-y-2">
                {(editingClient.post_logout_redirect_uris || []).map((uri, index) => (
                  <div key={index} className="flex gap-2">
                    <input
                      type="url"
                      value={uri}
                      onChange={(e) => {
                        const newUris = [...(editingClient.post_logout_redirect_uris || [])];
                        newUris[index] = e.target.value;
                        setEditingClient({ ...editingClient, post_logout_redirect_uris: newUris });
                      }}
                      placeholder="https://example.com/logout"
                      className="flex-1 bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-white/20 transition-all font-mono"
                    />
                    <button
                      type="button"
                      onClick={() =>
                        setEditingClient({
                          ...editingClient,
                          post_logout_redirect_uris: (
                            editingClient.post_logout_redirect_uris || []
                          ).filter((_, i) => i !== index),
                        })
                      }
                      className="p-2.5 text-gray-500 hover:text-red-500 hover:bg-red-500/[0.08] rounded-lg border border-gray-200 dark:border-white/[0.08] transition-colors"
                    >
                      <Minus className="w-4 h-4" />
                    </button>
                  </div>
                ))}
                <button
                  type="button"
                  onClick={() =>
                    setEditingClient({
                      ...editingClient,
                      post_logout_redirect_uris: [
                        ...(editingClient.post_logout_redirect_uris || []),
                        "",
                      ],
                    })
                  }
                  className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-dashed border-gray-300 dark:border-white/[0.12] hover:border-gray-400 dark:hover:border-white/20 rounded-lg transition-colors"
                >
                  <Plus className="w-3.5 h-3.5" />
                  添加登出回调
                </button>
              </div>
            </div>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={editingClient.is_active}
                onChange={(e) =>
                  setEditingClient({ ...editingClient, is_active: e.target.checked })
                }
                className="w-4 h-4 rounded border-gray-200 dark:border-white/[0.12] bg-gray-50 dark:bg-white/[0.04] text-white focus:ring-white/20"
              />
              <span className="text-sm text-gray-600 dark:text-gray-400">启用应用</span>
            </label>

            <label className="flex items-start gap-2 cursor-pointer rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-50 dark:bg-white/[0.03] px-3 py-3">
              <input
                type="checkbox"
                checked={editingClient.enable_silent_authorize}
                onChange={(e) =>
                  setEditingClient({ ...editingClient, enable_silent_authorize: e.target.checked })
                }
                className="mt-0.5 w-4 h-4 rounded border-gray-200 dark:border-white/[0.12] bg-gray-50 dark:bg-white/[0.04] text-white focus:ring-white/20"
              />
              <span>
                <span className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  启用无感授权
                </span>
                <span className="block text-xs text-gray-500 dark:text-gray-500 mt-0.5">
                  已登录且已同意过的用户，可直接返回 code，不再显示授权页
                </span>
              </span>
            </label>
          </form>
        </Modal>
      )}

      {/* Delete Modal */}
      {deletingClient && (
        <Modal
          isOpen={!!deletingClient}
          onClose={() => setDeletingClient(null)}
          title="删除应用"
          description={`确定要删除应用 "${deletingClient.name}" 吗？此操作不可恢复。`}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeletingClient(null)}>
                取消
              </Button>
              <Button variant="danger" onClick={handleDelete} loading={deleting}>
                删除
              </Button>
            </>
          }
        >
          <div className="p-3 bg-red-500/[0.08] border border-red-500/[0.16] rounded-lg">
            <p className="text-sm text-red-400">
              警告：删除应用后，所有使用该应用的客户端将无法继续登录。
            </p>
          </div>
        </Modal>
      )}

      {/* Reset Secret Confirm Modal */}
      {resettingClient && (
        <Modal
          isOpen={!!resettingClient}
          onClose={() => setResettingClient(null)}
          title="重置密钥"
          description={`确定要重置 "${resettingClient.name}" 的密钥吗？旧密钥将立即失效。`}
          footer={
            <>
              <Button variant="ghost" onClick={() => setResettingClient(null)}>
                取消
              </Button>
              <Button variant="danger" onClick={handleResetSecret} loading={resetting}>
                确认重置
              </Button>
            </>
          }
        >
          <div className="p-3 bg-amber-500/[0.08] border border-amber-500/[0.16] rounded-lg">
            <p className="text-sm text-amber-400">需要使用新密钥更新所有客户端配置。</p>
          </div>
        </Modal>
      )}

      {/* Secret Display Modal */}
      {newClientSecret && (
        <Modal
          isOpen={!!newClientSecret}
          onClose={() => setNewClientSecret(null)}
          title="新建客户端密钥"
          description={`${newClientSecret.client.name} · 仅展示一次`}
          size="xl"
          footer={<Button onClick={() => setNewClientSecret(null)}>我已复制并保存</Button>}
        >
          <div className="space-y-4">
            <div className="rounded-2xl border border-amber-500/25 bg-amber-500/[0.08] p-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="text-sm text-amber-300 font-semibold mb-1">请立即复制并保存</p>
                  <p className="text-xs text-amber-300/80">
                    密钥只显示一次，关闭后无法再次查看原文。
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() =>
                    handleCopy(
                      `CLIENT_ID=${newClientSecret.client.client_id}\nCLIENT_SECRET=${newClientSecret.secret}`,
                      "secret-bundle",
                    )
                  }
                  className="inline-flex items-center gap-1.5 rounded-md border border-amber-400/30 px-2.5 py-1.5 text-xs text-amber-200 hover:text-amber-100 hover:bg-amber-400/10 transition-colors"
                >
                  {copiedField === "secret-bundle" ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                  ) : (
                    <Copy className="w-3.5 h-3.5" />
                  )}
                  复制全部
                </button>
              </div>
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="rounded-2xl border border-gray-200 dark:border-white/[0.06] bg-gray-50 dark:bg-black/20 p-4">
                <div className="flex items-center justify-between gap-3 mb-2">
                  <div>
                    <p className="text-xs font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                      Client ID
                    </p>
                    <p className="text-[11px] text-gray-500 mt-1">公开标识，可用于 OIDC 配置</p>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleCopy(newClientSecret.client.client_id, "create-client-id")}
                    className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-white/[0.08] px-2.5 py-1.5 text-xs text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-white dark:hover:bg-white/[0.04] transition-colors"
                  >
                    {copiedField === "create-client-id" ? (
                      <Check className="w-3.5 h-3.5 text-emerald-500" />
                    ) : (
                      <Copy className="w-3.5 h-3.5" />
                    )}
                    复制 ID
                  </button>
                </div>
                <code className="block rounded-xl border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/30 px-4 py-4 text-sm font-mono text-gray-900 dark:text-white break-all leading-6">
                  {newClientSecret.client.client_id}
                </code>
              </div>

              <div className="rounded-2xl border border-gray-200 dark:border-white/[0.06] bg-gray-50 dark:bg-black/20 p-4">
                <div className="flex items-center justify-between gap-3 mb-2">
                  <div>
                    <p className="text-xs font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider">
                      Client Secret
                    </p>
                    <p className="text-[11px] text-gray-500 mt-1">敏感信息，仅此处可复制原始值</p>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleCopy(newClientSecret.secret, "secret")}
                    className="inline-flex items-center gap-1.5 rounded-md border border-gray-200 dark:border-white/[0.08] px-2.5 py-1.5 text-xs text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white hover:bg-white dark:hover:bg-white/[0.04] transition-colors"
                  >
                    {copiedField === "secret" ? (
                      <Check className="w-3.5 h-3.5 text-emerald-500" />
                    ) : (
                      <Copy className="w-3.5 h-3.5" />
                    )}
                    复制密钥
                  </button>
                </div>
                <code className="block rounded-xl border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-black/30 px-4 py-4 text-sm font-mono text-gray-900 dark:text-white break-all leading-6">
                  {newClientSecret.secret}
                </code>
              </div>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
