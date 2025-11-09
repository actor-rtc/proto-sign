#!/usr/bin/env bash
set -euo pipefail

# proto-sign Release Script
# 用于自动化发布 proto-sign 到 crates.io

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# 显示使用方法
usage() {
    cat <<EOF
用法: $0 <版本类型> [选项]

版本类型:
  patch    递增补丁版本 (0.1.0 -> 0.1.1)
  minor    递增次版本 (0.1.0 -> 0.2.0)
  major    递增主版本 (0.1.0 -> 1.0.0)
  <版本号> 直接指定版本号 (如 1.2.3)

选项:
  --dry-run    只执行验证，不实际发布
  --no-verify  跳过测试（不推荐）
  --help       显示此帮助信息

示例:
  $0 patch                 # 发布补丁版本
  $0 minor --dry-run       # 测试次版本发布流程
  $0 1.0.0                 # 直接发布 1.0.0 版本
EOF
    exit 1
}

# 解析参数
VERSION_TYPE=""
DRY_RUN=false
NO_VERIFY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        patch|minor|major)
            VERSION_TYPE="$1"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-verify)
            NO_VERIFY=true
            shift
            ;;
        --help)
            usage
            ;;
        [0-9]*.[0-9]*.[0-9]*)
            VERSION_TYPE="$1"
            shift
            ;;
        *)
            log_error "未知参数: $1"
            usage
            ;;
    esac
done

if [[ -z "$VERSION_TYPE" ]]; then
    log_error "请指定版本类型"
    usage
fi

# 获取当前版本
get_current_version() {
    grep '^version = ' Cargo.toml | head -n1 | sed -E 's/version = "(.*)"/\1/'
}

# 递增版本号
increment_version() {
    local version=$1
    local type=$2

    IFS='.' read -r major minor patch <<< "$version"

    case $type in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            echo "$type"
            ;;
    esac
}

# 检查 Git 状态
check_git_status() {
    log_info "检查 Git 状态..."

    if [[ -n $(git status --porcelain) ]]; then
        log_error "工作目录不干净，请先提交或暂存更改"
        git status --short
        exit 1
    fi

    # 确保在正确的分支
    local branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$branch" != "main" ]]; then
        log_warn "当前分支不是 main (当前: $branch)"
        read -p "是否继续? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi

    log_success "Git 状态检查通过"
}

# 更新版本号
update_version() {
    local new_version=$1

    log_info "更新版本号: $CURRENT_VERSION -> $new_version"

    sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$new_version\"/" Cargo.toml
    rm -f Cargo.toml.bak

    log_success "版本号已更新"
}

# 运行测试
run_tests() {
    if [[ "$NO_VERIFY" == true ]]; then
        log_warn "跳过测试（--no-verify）"
        return
    fi

    log_info "运行测试..."
    cargo test --all-features
    log_success "测试通过"
}

# 验证发布
verify_publish() {
    log_info "验证发布配置..."
    cargo publish --dry-run --allow-dirty
    log_success "发布验证通过"
}

# 发布到 crates.io
publish_crate() {
    if [[ "$DRY_RUN" == true ]]; then
        log_warn "Dry-run 模式，跳过实际发布"
        return
    fi

    log_info "发布到 crates.io..."
    cargo publish
    log_success "已发布到 crates.io"
}

# 创建 Git 标签
create_git_tag() {
    local version=$1
    local tag="v$version"

    if [[ "$DRY_RUN" == true ]]; then
        log_warn "Dry-run 模式，跳过 Git 标签"
        return
    fi

    log_info "创建 Git 标签: $tag"

    # 提交版本变更
    git add Cargo.toml Cargo.lock
    git commit -m "Release version $version"

    # 创建标签
    git tag -a "$tag" -m "Release $version"

    # 推送到远程
    git push origin HEAD
    git push origin "$tag"

    log_success "Git 标签已创建并推送: $tag"
}

# 恢复更改
rollback() {
    log_warn "发布失败，恢复更改..."
    git checkout -- Cargo.toml
    rm -f Cargo.toml.bak
}

# 主流程
main() {
    log_info "开始 proto-sign 发布流程"
    log_info "========================================"

    # 获取版本信息
    CURRENT_VERSION=$(get_current_version)
    NEW_VERSION=$(increment_version "$CURRENT_VERSION" "$VERSION_TYPE")

    log_info "当前版本: $CURRENT_VERSION"
    log_info "目标版本: $NEW_VERSION"

    if [[ "$DRY_RUN" == true ]]; then
        log_warn "Dry-run 模式：只验证，不实际发布"
    fi

    echo
    read -p "确认发布版本 $NEW_VERSION? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warn "已取消发布"
        exit 0
    fi

    # 执行发布流程
    trap rollback ERR

    check_git_status
    update_version "$NEW_VERSION"

    log_info "更新 Cargo.lock..."
    cargo update -p proto-sign

    run_tests
    verify_publish
    publish_crate
    create_git_tag "$NEW_VERSION"

    trap - ERR

    echo
    log_success "========================================"
    log_success "proto-sign $NEW_VERSION 发布完成！"
    log_success "========================================"

    if [[ "$DRY_RUN" == false ]]; then
        log_info "Crates.io: https://crates.io/crates/proto-sign"
        log_info "稍等几分钟后即可使用新版本"
    fi
}

main
