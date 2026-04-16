import { useState, useEffect } from "react";
import { Tags, Search, Plus, Users, Trash2, Edit } from "lucide-react";
import { adminApi } from "#src/api/admin";
import { useApi } from "#src/hooks";
import { Card, Button, Input, Modal, EmptyState, useToast } from "#src/components/ui";
import { getErrorMessage } from "#src/api/client";

// Animation keyframes
const fadeIn = `@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }`;
const slideUp = `@keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }`;

interface Group {
  id: string;
  name: string;
  description: string;
  member_count: number;
  created_at: string;
}

export function AdminGroups() {
  const [groups, setGroups] = useState<Group[]>([]);
  const [filteredGroups, setFilteredGroups] = useState<Group[]>([]);
  const [search, setSearch] = useState("");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [deletingGroup, setDeletingGroup] = useState<Group | null>(null);
  const { addToast } = useToast();

  // Form states
  const [formData, setFormData] = useState({
    name: "",
    description: "",
  });

  // Load groups
  useEffect(() => {
    loadGroups();
  }, []);

  // Filter groups
  useEffect(() => {
    const filtered = groups.filter(
      (g) =>
        g.name.toLowerCase().includes(search.toLowerCase()) ||
        g.description.toLowerCase().includes(search.toLowerCase()),
    );
    setFilteredGroups(filtered);
  }, [groups, search]);

  const loadGroups = async () => {
    try {
      const data = (await adminApi.getGroups()) as Group[];
      setGroups(data);
    } catch (err) {
      addToast({
        type: "error",
        title: "加载失败",
        message: getErrorMessage(err),
      });
    }
  };

  const { loading: creating, execute: createGroup } = useApi(adminApi.createGroup, {
    successMessage: "用户组创建成功",
    onSuccess: () => {
      setShowCreateModal(false);
      setFormData({ name: "", description: "" });
      loadGroups();
    },
  });

  const { loading: updating, execute: updateGroup } = useApi(
    (id: string, data: Record<string, unknown>) => adminApi.updateGroup(id, data),
    {
      successMessage: "用户组更新成功",
      onSuccess: () => {
        setEditingGroup(null);
        loadGroups();
      },
    },
  );

  const { loading: deleting, execute: deleteGroup } = useApi(
    (id: string) => adminApi.deleteGroup(id),
    {
      successMessage: "用户组已删除",
      onSuccess: () => {
        setDeletingGroup(null);
        loadGroups();
      },
    },
  );

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    await createGroup(formData);
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingGroup) return;
    await updateGroup(editingGroup.id, {
      name: editingGroup.name,
      description: editingGroup.description,
    });
  };

  const handleDelete = async () => {
    if (!deletingGroup) return;
    await deleteGroup(deletingGroup.id);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString("zh-CN");
  };

  return (
    <div className="space-y-5" style={{ animation: "slideUp 0.3s ease-out" }}>
      <style>
        {fadeIn}
        {slideUp}
      </style>
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">用户组管理</h1>
          <p className="text-sm text-gray-500 mt-0.5">管理用户组和成员关系</p>
        </div>
        <Button onClick={() => setShowCreateModal(true)} size="sm">
          <Plus className="w-4 h-4" />
          添加用户组
        </Button>
      </div>

      {/* Search */}
      <Card padding="sm">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600" />
          <input
            type="text"
            placeholder="搜索用户组..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent pl-9 pr-4 py-2 text-sm text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-gray-700 focus:outline-none"
          />
        </div>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-3">
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-900 dark:text-white">{groups.length}</p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">
            用户组
          </p>
        </Card>
        <Card className="text-center py-4">
          <p className="text-2xl font-bold text-gray-600 dark:text-gray-300">
            {groups.reduce((sum, g) => sum + g.member_count, 0)}
          </p>
          <p className="text-[11px] text-gray-500 dark:text-gray-600 uppercase tracking-wider mt-1">
            成员关系数
          </p>
        </Card>
      </div>

      {/* Grid */}
      {filteredGroups.length === 0 ? (
        <Card padding="lg">
          <EmptyState
            icon={<Tags className="w-6 h-6" />}
            title="暂无用户组"
            description="还没有任何用户组，点击上方按钮添加"
          />
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {filteredGroups.map((group) => (
            <Card key={group.id} hover>
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  <div className="w-10 h-10 rounded-lg bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.06] flex items-center justify-center flex-shrink-0">
                    <Tags className="w-5 h-5 text-gray-600 dark:text-gray-400" />
                  </div>
                  <div>
                    <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                      {group.name}
                    </h3>
                    <p className="text-xs text-gray-500 mt-0.5">{group.description}</p>
                    <div className="flex items-center gap-3 mt-2 text-xs text-gray-500 dark:text-gray-600">
                      <div className="flex items-center gap-1">
                        <Users className="w-3.5 h-3.5" />
                        {group.member_count} 成员
                      </div>
                      <span>创建于 {formatDate(group.created_at)}</span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1">
                  <button
                    onClick={() => setEditingGroup(group)}
                    className="p-1.5 text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/[0.04] rounded transition-colors"
                    title="编辑"
                  >
                    <Edit className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => setDeletingGroup(group)}
                    className="p-1.5 text-gray-500 hover:text-red-400 hover:bg-red-500/[0.08] rounded transition-colors"
                    title="删除"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Create Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="添加用户组"
        description="创建新的用户组"
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
            label="组名称"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="例如: developers"
            required
          />
          <div>
            <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
              描述
            </label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="用户组描述"
              rows={3}
              className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 dark:placeholder:text-gray-600 focus:outline-none focus:ring-1 focus:ring-white/20 focus:border-white/20 transition-all resize-none"
            />
          </div>
        </form>
      </Modal>

      {/* Edit Modal */}
      {editingGroup && (
        <Modal
          isOpen={!!editingGroup}
          onClose={() => setEditingGroup(null)}
          title="编辑用户组"
          footer={
            <>
              <Button variant="ghost" onClick={() => setEditingGroup(null)}>
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
              label="组名称"
              value={editingGroup.name}
              onChange={(e) => setEditingGroup({ ...editingGroup, name: e.target.value })}
            />
            <div>
              <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                描述
              </label>
              <textarea
                value={editingGroup.description}
                onChange={(e) => setEditingGroup({ ...editingGroup, description: e.target.value })}
                rows={3}
                className="w-full bg-gray-100 dark:bg-white/[0.04] border border-gray-200 dark:border-white/[0.08] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white placeholder:text-gray-500 dark:placeholder:text-gray-600 focus:outline-none focus:ring-1 focus:ring-white/20 focus:border-white/20 transition-all resize-none"
              />
            </div>
          </form>
        </Modal>
      )}

      {/* Delete Modal */}
      {deletingGroup && (
        <Modal
          isOpen={!!deletingGroup}
          onClose={() => setDeletingGroup(null)}
          title="删除用户组"
          description={`确定要删除用户组 "${deletingGroup.name}" 吗？此操作不可恢复。`}
          footer={
            <>
              <Button variant="ghost" onClick={() => setDeletingGroup(null)}>
                取消
              </Button>
              <Button onClick={handleDelete} loading={deleting} variant="danger">
                删除
              </Button>
            </>
          }
        >
          <div className="p-3 bg-red-500/[0.08] border border-red-500/[0.16] rounded-lg">
            <p className="text-sm text-red-400">
              警告：删除用户组将影响 {deletingGroup.member_count} 个成员。
            </p>
          </div>
        </Modal>
      )}
    </div>
  );
}
