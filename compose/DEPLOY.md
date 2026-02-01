# Convertor 部署指南

## 一、准备工作

配置 GitHub Secrets，添加 SSH 私钥：

- **名称**: `CONVERTOR_SSH_KEY`
- **内容**: `~/.ssh/convertor_ed25519` 的私钥内容

```bash
cat ~/.ssh/convertor_ed25519 | pbcopy
```

在 GitHub 仓库设置: **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

完成后，推送 tag 或手动触发 workflow，剩下的全部自动完成！

## 四、配置 Nginx Proxy Manager

### 41: 推送 Tag（推荐）\*\*

```bash
git tag v1.0.0
git push origin v1.0.0
```

**方式 2: 手动触发**

在 GitHub 仓库: **Actions** → **构建并推送镜像** → **Run workflow** → 勾选 "是否部署到 node-vultr-1"

## 三、自动化流程

Workflow 会自动完成：

1. **构建阶段**：
    - 构建多平台镜像（amd64 + arm64）
    - 推送到 GHCR

2. **部署阶段**：
    - 检查并安装 Docker（如未安装）
    - 上传 compose.yaml
    - 拉取最新镜像
    - 启动/重启服务
    - 清理旧镜像

## 四、配置 Nginx Proxy Manager

### 2.1 登录管理界面

访问: `http://167.179.111.233:81`

- 默认账号: `admin@example.com`
- 默认密码: `changeme`

**⚠️ 首次登录后请立即修改密码！**

### 4.2 配置反向代理

1. 点击 **Proxy Hosts** → **Add Proxy Host**
2. 填写配置:
    - **Domain Names**: 你的域名（如 `conv.example.com`）
    - **Scheme**: `http`
    - **Forward Hostname / IP**: `convd`
    - **Forward Port**: `8080`
    - **Cache Assets**: ✅ 开启
    - **Block Common Exploits**: ✅ 开启
    - **Websockets Support**: ✅ 开启

3. 配置 SSL:
    - 切换到 **SSL** 标签
    - **SSL Certificate**: 选择 **Request a new SSL Certificate**
    - **Force SSL**: ✅ 开启
    - **Email**: 填写你的邮箱
    - **Use a DNS Challenge**: 根据需要选择
    - 点击 **Save**

## 五、监控与管理

### 5.1 查看部署状态

```bash
ssh node-vultr-1 'cd /opt/convertor && docker compose ps'
```

### 5.2 查看服务日志

```bash
# 查看 convd 日志
ssh node-vultr-1 'cd /opt/convertor && docker compose logs -f convd'

# 查看 Nginx Proxy Manager 日志
ssh node-vultr-1 'cd /opt/convertor && docker compose logs -f nginx-proxy-manager'
```

### 5.2 重启服务

```bash
# 重启 convd
ssh node-vultr-1 'cd /opt/convertor && docker compose restart convd'

# 重启所有服务
ssh node-vultr-1 'cd /opt/convertor && docker compose restart'
```

### 5.3 检查健康状态

```bash
# 检查 convd 健康状态
ssh node-vultr-1 'docker exec convd wget -q --spider http://localhost:8080/health && echo "健康" || echo "不健康"'
```

## 六、常用命令

```bash
# 进入 convd 容器
ssh node-vultr-1 'docker exec -it convd sh'

# 查看 convd 版本
ssh node-vultr-1 'docker exec convd /app/convd tag'

# 查看运行状态
ssh node-vultr-1 'cd /opt/convertor && docker compose ps'

# 停止所有服务
ssh node-vultr-1 'cd /opt/convertor && docker compose down'

# 启动所有服务
ssh node-vultr-1 'cd /opt/convertor && docker compose up -d'
```

## 七、数据持久化

以下数据会被持久化保存：

- **npm-data**: Nginx Proxy Manager 配置和数据库
- **npm-letsencrypt**: SSL 证书
- **convd-data**: Convertor 配置文件

备份数据：

```bash
ssh node-vultr-1 'cd /opt/convertor && docker compose down'
ssh node-vultr-1 'tar -czf /tmp/convertor-backup-$(date +%Y%m%d).tar.gz -C /var/lib/docker/volumes .'
scp node-vultr-1:/tmp/convertor-backup-*.tar.gz ./backups/
```

34
