"""Access policy definitions and enforcement."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class Permission(Enum):
    DOMAIN_REGISTER = "domain:register"
    DOMAIN_Update = "domain:update"
    DOMAIN_Delete = "domain:delete"
    FEED_READ = "feed:read"
    FEED_WRITE = "feed:write"
    MESSAGE_SEND = "message:send"
    MESSAGE_READ = "message:read"
    IDENTITY_READ = "identity:read"
    IDENTITY_UPDATE = "identity:update"
    ADMIN = "admin"


class Role(Enum):
    OWNER = "owner"
    MEMBER = "member"
    GUEST = "guest"


# Role → permissions mapping
ROLE_PERMISSIONS: dict[Role, set[Permission]] = {
    Role.OWNER: set(Permission),  # All permissions
    Role.MEMBER: {
        Permission.DOMAIN_REGISTER,
        Permission.DOMAIN_Update,
        Permission.FEED_READ,
        Permission.FEED_WRITE,
        Permission.MESSAGE_SEND,
        Permission.MESSAGE_READ,
        Permission.IDENTITY_READ,
        Permission.IDENTITY_UPDATE,
    },
    Role.GUEST: {
        Permission.FEED_READ,
        Permission.MESSAGE_READ,
        Permission.IDENTITY_READ,
    },
}


@dataclass
class AccessPolicy:
    """Represents an access policy rule."""
    resource: str
    action: str
    allowed_roles: list[Role] = field(default_factory=lambda: [Role.OWNER, Role.MEMBER])
    conditions: dict[str, Any] = field(default_factory=dict)


# Default access policies
DEFAULT_POLICIES: list[AccessPolicy] = [
    AccessPolicy("domain", "register", [Role.OWNER, Role.MEMBER]),
    AccessPolicy("domain", "update", [Role.OWNER]),
    AccessPolicy("domain", "delete", [Role.OWNER]),
    AccessPolicy("feed", "read", [Role.OWNER, Role.MEMBER, Role.GUEST]),
    AccessPolicy("feed", "write", [Role.OWNER, Role.MEMBER]),
    AccessPolicy("message", "send", [Role.OWNER, Role.MEMBER]),
    AccessPolicy("message", "read", [Role.OWNER, Role.MEMBER]),
    AccessPolicy("identity", "read", [Role.OWNER, Role.MEMBER, Role.GUEST]),
    AccessPolicy("identity", "update", [Role.OWNER]),
    AccessPolicy("admin", "*", [Role.OWNER]),
]


def check_permission(role: Role, resource: str, action: str) -> bool:
    """Check if a role has permission for a resource action."""
    # Direct permission check
    for policy in DEFAULT_POLICIES:
        if policy.resource == resource and (policy.action == action or policy.action == "*"):
            if role in policy.allowed_roles:
                return True

    # Role-based fallback
    role_perms = ROLE_PERMISSIONS.get(role, set())
    perm_name = f"{resource}:{action}"
    for perm in role_perms:
        if perm.value == perm_name:
            return True

    return False


def get_permissions(role: Role) -> set[str]:
    """Get all permission strings for a role."""
    return {p.value for p in ROLE_PERMISSIONS.get(role, set())}
