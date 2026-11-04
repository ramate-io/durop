from pkgutil import extend_path
__path__ = extend_path(__path__, __name__)

import abc
import builtins
import datetime
import enum
import importlib as _importlib
import typing

import jsii
import publication
import typing_extensions

from jsii._type_checking import cached_type_hints, check_type


from ._jsii import *

__all__ = [
    "acl",
    "aws_external_id",
    "contacts",
    "data_tailscale4_via6",
    "data_tailscale_acl",
    "data_tailscale_device",
    "data_tailscale_devices",
    "data_tailscale_service",
    "data_tailscale_user",
    "data_tailscale_users",
    "device_authorization",
    "device_key",
    "device_subnet_routes",
    "device_tags",
    "dns_configuration",
    "dns_nameservers",
    "dns_preferences",
    "dns_search_paths",
    "dns_split_nameservers",
    "federated_identity",
    "logstream_configuration",
    "oauth_client",
    "posture_integration",
    "provider",
    "service",
    "tailnet_key",
    "tailnet_settings",
    "webhook",
]

# Type-checking-only imports for static analyzers (pyright/mypy).
# At runtime TYPE_CHECKING is False, preserving lazy loading.
if typing.TYPE_CHECKING:
    pass
                                                                                                                
publication.publish()

_SUBMODULES = {
    "acl",
    "aws_external_id",
    "contacts",
    "data_tailscale_acl",
    "data_tailscale_device",
    "data_tailscale_devices",
    "data_tailscale_service",
    "data_tailscale_user",
    "data_tailscale_users",
    "data_tailscale4_via6",
    "device_authorization",
    "device_key",
    "device_subnet_routes",
    "device_tags",
    "dns_configuration",
    "dns_nameservers",
    "dns_preferences",
    "dns_search_paths",
    "dns_split_nameservers",
    "federated_identity",
    "logstream_configuration",
    "oauth_client",
    "posture_integration",
    "provider",
    "service",
    "tailnet_key",
    "tailnet_settings",
    "webhook",
}

def __getattr__(name: str) -> object:
    if name in _SUBMODULES:
        mod = _importlib.import_module(f".{name}", __name__)
        globals()[name] = mod
        return mod
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

def __dir__() -> "list[str]":
    return [*__all__, *_SUBMODULES]

import sys as _sys
setattr(_sys.modules[__name__], "__getattr__", __getattr__)
setattr(_sys.modules[__name__], "__dir__", __dir__)
