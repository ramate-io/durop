from pkgutil import extend_path
__path__ = extend_path(__path__, __name__)

import abc
import builtins
import datetime
import enum
import typing

import jsii
import publication
import typing_extensions

from jsii._type_checking import cached_type_hints, check_type


import cdktn._jsii
import constructs._jsii

_SUBMODULE_FQN_MAP = {
    "tailscale.acl": "tailscale.acl",
    "tailscale.awsExternalId": "tailscale.aws_external_id",
    "tailscale.contacts": "tailscale.contacts",
    "tailscale.dataTailscale4Via6": "tailscale.data_tailscale4_via6",
    "tailscale.dataTailscaleAcl": "tailscale.data_tailscale_acl",
    "tailscale.dataTailscaleDevice": "tailscale.data_tailscale_device",
    "tailscale.dataTailscaleDevices": "tailscale.data_tailscale_devices",
    "tailscale.dataTailscaleService": "tailscale.data_tailscale_service",
    "tailscale.dataTailscaleUser": "tailscale.data_tailscale_user",
    "tailscale.dataTailscaleUsers": "tailscale.data_tailscale_users",
    "tailscale.deviceAuthorization": "tailscale.device_authorization",
    "tailscale.deviceKey": "tailscale.device_key",
    "tailscale.deviceSubnetRoutes": "tailscale.device_subnet_routes",
    "tailscale.deviceTags": "tailscale.device_tags",
    "tailscale.dnsConfiguration": "tailscale.dns_configuration",
    "tailscale.dnsNameservers": "tailscale.dns_nameservers",
    "tailscale.dnsPreferences": "tailscale.dns_preferences",
    "tailscale.dnsSearchPaths": "tailscale.dns_search_paths",
    "tailscale.dnsSplitNameservers": "tailscale.dns_split_nameservers",
    "tailscale.federatedIdentity": "tailscale.federated_identity",
    "tailscale.logstreamConfiguration": "tailscale.logstream_configuration",
    "tailscale.oauthClient": "tailscale.oauth_client",
    "tailscale.postureIntegration": "tailscale.posture_integration",
    "tailscale.provider": "tailscale.provider",
    "tailscale.service": "tailscale.service",
    "tailscale.tailnetKey": "tailscale.tailnet_key",
    "tailscale.tailnetSettings": "tailscale.tailnet_settings",
    "tailscale.webhook": "tailscale.webhook",
}

__jsii_assembly__ = jsii.JSIIAssembly.load(
    "tailscale", "0.0.0", __name__[0:-6], "tailscale@0.0.0.jsii.tgz"
)

__all__ = [
    "__jsii_assembly__",
]

publication.publish()
