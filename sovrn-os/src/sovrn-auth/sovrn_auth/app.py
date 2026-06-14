"""Auth service FastAPI application."""

from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel

from sovrn_auth.policy import Role, check_permission, get_permissions

app = FastAPI(title="Sovrn Auth Service", version="0.1.0")


class PermissionCheck(BaseModel):
    role: str
    resource: str
    action: str


class RoleInfo(BaseModel):
    role: str


@app.get("/health")
async def health():
    return {"status": "ok", "service": "sovrn-auth"}


@app.post("/check")
async def check_perm(body: PermissionCheck):
    """Check if a role has permission for a resource action."""
    try:
        role = Role(body.role)
    except ValueError:
        raise HTTPException(status_code=400, detail=f"Invalid role: {body.role}")

    allowed = check_permission(role, body.resource, body.action)
    return {"allowed": allowed, "role": body.role, "resource": body.resource, "action": body.action}


@app.get("/permissions/{role}")
async def get_perms(role: str):
    """Get all permissions for a role."""
    try:
        r = Role(role)
    except ValueError:
        raise HTTPException(status_code=400, detail=f"Invalid role: {role}")
    return {"role": role, "permissions": list(get_permissions(r))}
