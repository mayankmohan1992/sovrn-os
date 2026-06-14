"""JWT authentication for Sovrn OS API."""

import hashlib
import hmac
import secrets
import time
from dataclasses import dataclass

import jwt


@dataclass
class TokenData:
    public_key: str
    domain: str | None
    exp: float
    iat: float


def create_jwt(config, public_key: str, domain: str | None = None) -> str:
    """Create a JWT token for the given identity."""
    now = time.time()
    payload = {
        "sub": public_key,
        "domain": domain,
        "iat": int(now),
        "exp": int(now + config.jwt_expiry_hours * 3600),
        "iss": "sovrn-os",
    }
    return jwt.encode(payload, config.jwt_secret, algorithm=config.jwt_algorithm)


def verify_jwt(config, token: str) -> TokenData | None:
    """Verify and decode a JWT token."""
    try:
        payload = jwt.decode(
            token,
            config.jwt_secret,
            algorithms=[config.jwt_algorithm],
            options={"require": ["sub", "exp", "iat"]},
        )
        return TokenData(
            public_key=payload["sub"],
            domain=payload.get("domain"),
            exp=payload["exp"],
            iat=payload["iat"],
        )
    except (jwt.InvalidTokenError, jwt.DecodeError):
        return None


def generate_device_token() -> str:
    """Generate a random device token for OOBE provisioning."""
    return secrets.token_urlsafe(32)


def hash_password(password: str, salt: str | None = None) -> tuple[str, str]:
    """Hash a password with a salt using PBKDF2."""
    if salt is None:
        salt = secrets.token_hex(16)
    key = hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), 600000)
    return key.hex(), salt


def verify_password(password: str, stored_hash: str, salt: str) -> bool:
    """Verify a password against stored hash."""
    key = hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), 600000)
    return hmac.compare_digest(key.hex(), stored_hash)
