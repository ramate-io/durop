r'''
# `tailscale_tailnet_settings`

Refer to the OpenTofu Registry for docs: [`tailscale_tailnet_settings`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings).
'''
from __future__ import annotations

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


from .._jsii import *

class _LazyImport:
    def __init__(self, module_name: str) -> None:
        self._module_name = module_name
        self._module: typing.Any = None
    def __getattr__(self, name: str) -> typing.Any:
        if self._module is None:
            import importlib
            self._module = importlib.import_module(self._module_name)
        return getattr(self._module, name)

if typing.TYPE_CHECKING:

    import cdktn as _cdktn_78ede62e
    import constructs as _constructs_77d1e7e8
else:

    _cdktn_78ede62e = _LazyImport("cdktn")
    _constructs_77d1e7e8 = _LazyImport("constructs")


class TailnetSettings(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.tailnetSettings.TailnetSettings",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings tailscale_tailnet_settings}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        acls_external_link: typing.Optional[builtins.str] = None,
        acls_externally_managed_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_approval_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_auto_updates_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_key_duration_days: typing.Optional[jsii.Number] = None,
        https_enabled: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        network_flow_logging_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        posture_identity_collection_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        regional_routing_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        users_approval_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        users_role_allowed_to_join_external_tailnet: typing.Optional[builtins.str] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings tailscale_tailnet_settings} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param acls_external_link: Link to your external ACL definition or management system. Must be a valid URL. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_external_link TailnetSettings#acls_external_link}
        :param acls_externally_managed_on: Prevent users from editing policies in the admin console to avoid conflicts with external management workflows like GitOps or Terraform. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_externally_managed_on TailnetSettings#acls_externally_managed_on}
        :param devices_approval_on: Whether device approval is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_approval_on TailnetSettings#devices_approval_on}
        :param devices_auto_updates_on: Whether auto updates are enabled for devices that belong to this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_auto_updates_on TailnetSettings#devices_auto_updates_on}
        :param devices_key_duration_days: The key expiry duration for devices on this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_key_duration_days TailnetSettings#devices_key_duration_days}
        :param https_enabled: Whether provisioning of HTTPS certificates is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#https_enabled TailnetSettings#https_enabled}
        :param network_flow_logging_on: Whether network flow logs are enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#network_flow_logging_on TailnetSettings#network_flow_logging_on}
        :param posture_identity_collection_on: Whether identity collection is enabled for device posture integrations for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#posture_identity_collection_on TailnetSettings#posture_identity_collection_on}
        :param regional_routing_on: Whether regional routing is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#regional_routing_on TailnetSettings#regional_routing_on}
        :param users_approval_on: Whether user approval is enabled for this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_approval_on TailnetSettings#users_approval_on}
        :param users_role_allowed_to_join_external_tailnet: Which user roles are allowed to join external tailnets. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_role_allowed_to_join_external_tailnet TailnetSettings#users_role_allowed_to_join_external_tailnet}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__dbb71315ce324f6371c806951765b9d063631af8ed2172a8f0f88ac19fb959b5)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = TailnetSettingsConfig(
            acls_external_link=acls_external_link,
            acls_externally_managed_on=acls_externally_managed_on,
            devices_approval_on=devices_approval_on,
            devices_auto_updates_on=devices_auto_updates_on,
            devices_key_duration_days=devices_key_duration_days,
            https_enabled=https_enabled,
            network_flow_logging_on=network_flow_logging_on,
            posture_identity_collection_on=posture_identity_collection_on,
            regional_routing_on=regional_routing_on,
            users_approval_on=users_approval_on,
            users_role_allowed_to_join_external_tailnet=users_role_allowed_to_join_external_tailnet,
            connection=connection,
            count=count,
            depends_on=depends_on,
            for_each=for_each,
            lifecycle=lifecycle,
            provider=provider,
            provisioners=provisioners,
        )

        jsii.create(self.__class__, self, [scope, id, config])

    @jsii.member(jsii_name="generateConfigForImport")
    @builtins.classmethod
    def generate_config_for_import(
        cls,
        scope: "_constructs_77d1e7e8.Construct",
        import_to_id: builtins.str,
        import_from_id: builtins.str,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
    ) -> "_cdktn_78ede62e.ImportableResource":
        '''Generates CDKTN code for importing a TailnetSettings resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the TailnetSettings to import.
        :param import_from_id: The id of the existing TailnetSettings that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the TailnetSettings to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__143fdbd3465f8148dbc3428de1e381d2e2f4a7d4cc8b3f01eb10534d1863e187)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetAclsExternalLink")
    def reset_acls_external_link(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetAclsExternalLink", []))

    @jsii.member(jsii_name="resetAclsExternallyManagedOn")
    def reset_acls_externally_managed_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetAclsExternallyManagedOn", []))

    @jsii.member(jsii_name="resetDevicesApprovalOn")
    def reset_devices_approval_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetDevicesApprovalOn", []))

    @jsii.member(jsii_name="resetDevicesAutoUpdatesOn")
    def reset_devices_auto_updates_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetDevicesAutoUpdatesOn", []))

    @jsii.member(jsii_name="resetDevicesKeyDurationDays")
    def reset_devices_key_duration_days(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetDevicesKeyDurationDays", []))

    @jsii.member(jsii_name="resetHttpsEnabled")
    def reset_https_enabled(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetHttpsEnabled", []))

    @jsii.member(jsii_name="resetNetworkFlowLoggingOn")
    def reset_network_flow_logging_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetNetworkFlowLoggingOn", []))

    @jsii.member(jsii_name="resetPostureIdentityCollectionOn")
    def reset_posture_identity_collection_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetPostureIdentityCollectionOn", []))

    @jsii.member(jsii_name="resetRegionalRoutingOn")
    def reset_regional_routing_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetRegionalRoutingOn", []))

    @jsii.member(jsii_name="resetUsersApprovalOn")
    def reset_users_approval_on(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUsersApprovalOn", []))

    @jsii.member(jsii_name="resetUsersRoleAllowedToJoinExternalTailnet")
    def reset_users_role_allowed_to_join_external_tailnet(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUsersRoleAllowedToJoinExternalTailnet", []))

    @jsii.member(jsii_name="synthesizeAttributes")
    def _synthesize_attributes(self) -> typing.Mapping[builtins.str, typing.Any]:
        return typing.cast(typing.Mapping[builtins.str, typing.Any], jsii.invoke(self, "synthesizeAttributes", []))

    @jsii.member(jsii_name="synthesizeHclAttributes")
    def _synthesize_hcl_attributes(self) -> typing.Mapping[builtins.str, typing.Any]:
        return typing.cast(typing.Mapping[builtins.str, typing.Any], jsii.invoke(self, "synthesizeHclAttributes", []))

    @jsii.python.classproperty
    @jsii.member(jsii_name="tfResourceType")
    def TF_RESOURCE_TYPE(cls) -> builtins.str:
        return typing.cast(builtins.str, jsii.sget(cls, "tfResourceType"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="aclsExternalLinkInput")
    def acls_external_link_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "aclsExternalLinkInput"))

    @builtins.property
    @jsii.member(jsii_name="aclsExternallyManagedOnInput")
    def acls_externally_managed_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "aclsExternallyManagedOnInput"))

    @builtins.property
    @jsii.member(jsii_name="devicesApprovalOnInput")
    def devices_approval_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "devicesApprovalOnInput"))

    @builtins.property
    @jsii.member(jsii_name="devicesAutoUpdatesOnInput")
    def devices_auto_updates_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "devicesAutoUpdatesOnInput"))

    @builtins.property
    @jsii.member(jsii_name="devicesKeyDurationDaysInput")
    def devices_key_duration_days_input(self) -> typing.Optional[jsii.Number]:
        return typing.cast(typing.Optional[jsii.Number], jsii.get(self, "devicesKeyDurationDaysInput"))

    @builtins.property
    @jsii.member(jsii_name="httpsEnabledInput")
    def https_enabled_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "httpsEnabledInput"))

    @builtins.property
    @jsii.member(jsii_name="networkFlowLoggingOnInput")
    def network_flow_logging_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "networkFlowLoggingOnInput"))

    @builtins.property
    @jsii.member(jsii_name="postureIdentityCollectionOnInput")
    def posture_identity_collection_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "postureIdentityCollectionOnInput"))

    @builtins.property
    @jsii.member(jsii_name="regionalRoutingOnInput")
    def regional_routing_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "regionalRoutingOnInput"))

    @builtins.property
    @jsii.member(jsii_name="usersApprovalOnInput")
    def users_approval_on_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "usersApprovalOnInput"))

    @builtins.property
    @jsii.member(jsii_name="usersRoleAllowedToJoinExternalTailnetInput")
    def users_role_allowed_to_join_external_tailnet_input(
        self,
    ) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "usersRoleAllowedToJoinExternalTailnetInput"))

    @builtins.property
    @jsii.member(jsii_name="aclsExternalLink")
    def acls_external_link(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "aclsExternalLink"))

    @acls_external_link.setter
    def acls_external_link(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__fea4259eda6f308c157fb2cc165a6bdea20fd1f2db432dff41117a62d1c30346)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "aclsExternalLink", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="aclsExternallyManagedOn")
    def acls_externally_managed_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "aclsExternallyManagedOn"))

    @acls_externally_managed_on.setter
    def acls_externally_managed_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__36325c359a7c2bacb4e836f7404d909ec0094d67deddad595990c0e230055bf1)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "aclsExternallyManagedOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="devicesApprovalOn")
    def devices_approval_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "devicesApprovalOn"))

    @devices_approval_on.setter
    def devices_approval_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4e1dff6a145f0af1d71d9135a7c1901f80b9730e37ca6373456747e0d7bccb56)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "devicesApprovalOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="devicesAutoUpdatesOn")
    def devices_auto_updates_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "devicesAutoUpdatesOn"))

    @devices_auto_updates_on.setter
    def devices_auto_updates_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__dc843244c6537d44a9bf10b569021c1316c99e43dde36620577d72b66b69bae3)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "devicesAutoUpdatesOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="devicesKeyDurationDays")
    def devices_key_duration_days(self) -> jsii.Number:
        return typing.cast(jsii.Number, jsii.get(self, "devicesKeyDurationDays"))

    @devices_key_duration_days.setter
    def devices_key_duration_days(self, value: jsii.Number) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c323018adcd99f7b850070e73c9c1558e9ca9cb0212f925c5e802c58c1f0a2be)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "devicesKeyDurationDays", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="httpsEnabled")
    def https_enabled(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "httpsEnabled"))

    @https_enabled.setter
    def https_enabled(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__fa7a5c6d49efe9c57620033ec25814694e9628d9a80d86643fefbd08b21c1f32)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "httpsEnabled", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="networkFlowLoggingOn")
    def network_flow_logging_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "networkFlowLoggingOn"))

    @network_flow_logging_on.setter
    def network_flow_logging_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__346d4abfed08bcc5e3752edec17ee81f3aa89d3e7f46446ab4c13c9f307b757f)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "networkFlowLoggingOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="postureIdentityCollectionOn")
    def posture_identity_collection_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "postureIdentityCollectionOn"))

    @posture_identity_collection_on.setter
    def posture_identity_collection_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__8fd8cef20312361ec73c6348e29fcb2b20a4df8c86a6e2feb5af3160e83ea17b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "postureIdentityCollectionOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="regionalRoutingOn")
    def regional_routing_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "regionalRoutingOn"))

    @regional_routing_on.setter
    def regional_routing_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__ec49d64a54d0aec5cbf765702033d3197e7dd7f9c19139ab57654c90ae51cb2e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "regionalRoutingOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="usersApprovalOn")
    def users_approval_on(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "usersApprovalOn"))

    @users_approval_on.setter
    def users_approval_on(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__5e54502c4a6e59eda180c23187c8181f3de1db5651b89156eb37fefc851cfef9)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "usersApprovalOn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="usersRoleAllowedToJoinExternalTailnet")
    def users_role_allowed_to_join_external_tailnet(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "usersRoleAllowedToJoinExternalTailnet"))

    @users_role_allowed_to_join_external_tailnet.setter
    def users_role_allowed_to_join_external_tailnet(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__5552288a46b2d26de318d077d2fc16c8f26d7f5f595ef526409039354fd1cbeb)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "usersRoleAllowedToJoinExternalTailnet", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.tailnetSettings.TailnetSettingsConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "acls_external_link": "aclsExternalLink",
        "acls_externally_managed_on": "aclsExternallyManagedOn",
        "devices_approval_on": "devicesApprovalOn",
        "devices_auto_updates_on": "devicesAutoUpdatesOn",
        "devices_key_duration_days": "devicesKeyDurationDays",
        "https_enabled": "httpsEnabled",
        "network_flow_logging_on": "networkFlowLoggingOn",
        "posture_identity_collection_on": "postureIdentityCollectionOn",
        "regional_routing_on": "regionalRoutingOn",
        "users_approval_on": "usersApprovalOn",
        "users_role_allowed_to_join_external_tailnet": "usersRoleAllowedToJoinExternalTailnet",
    },
)
class TailnetSettingsConfig(_cdktn_78ede62e.TerraformMetaArguments):
    def __init__(
        self,
        *,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
        acls_external_link: typing.Optional[builtins.str] = None,
        acls_externally_managed_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_approval_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_auto_updates_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        devices_key_duration_days: typing.Optional[jsii.Number] = None,
        https_enabled: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        network_flow_logging_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        posture_identity_collection_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        regional_routing_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        users_approval_on: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        users_role_allowed_to_join_external_tailnet: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param acls_external_link: Link to your external ACL definition or management system. Must be a valid URL. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_external_link TailnetSettings#acls_external_link}
        :param acls_externally_managed_on: Prevent users from editing policies in the admin console to avoid conflicts with external management workflows like GitOps or Terraform. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_externally_managed_on TailnetSettings#acls_externally_managed_on}
        :param devices_approval_on: Whether device approval is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_approval_on TailnetSettings#devices_approval_on}
        :param devices_auto_updates_on: Whether auto updates are enabled for devices that belong to this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_auto_updates_on TailnetSettings#devices_auto_updates_on}
        :param devices_key_duration_days: The key expiry duration for devices on this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_key_duration_days TailnetSettings#devices_key_duration_days}
        :param https_enabled: Whether provisioning of HTTPS certificates is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#https_enabled TailnetSettings#https_enabled}
        :param network_flow_logging_on: Whether network flow logs are enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#network_flow_logging_on TailnetSettings#network_flow_logging_on}
        :param posture_identity_collection_on: Whether identity collection is enabled for device posture integrations for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#posture_identity_collection_on TailnetSettings#posture_identity_collection_on}
        :param regional_routing_on: Whether regional routing is enabled for the tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#regional_routing_on TailnetSettings#regional_routing_on}
        :param users_approval_on: Whether user approval is enabled for this tailnet. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_approval_on TailnetSettings#users_approval_on}
        :param users_role_allowed_to_join_external_tailnet: Which user roles are allowed to join external tailnets. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_role_allowed_to_join_external_tailnet TailnetSettings#users_role_allowed_to_join_external_tailnet}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__86c6dd20e39ed19251a083661ecb4fe2906d9fb7eab7c9c8f39fcfda282861b1)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument acls_external_link", value=acls_external_link, expected_type=type_hints["acls_external_link"])
            check_type(argname="argument acls_externally_managed_on", value=acls_externally_managed_on, expected_type=type_hints["acls_externally_managed_on"])
            check_type(argname="argument devices_approval_on", value=devices_approval_on, expected_type=type_hints["devices_approval_on"])
            check_type(argname="argument devices_auto_updates_on", value=devices_auto_updates_on, expected_type=type_hints["devices_auto_updates_on"])
            check_type(argname="argument devices_key_duration_days", value=devices_key_duration_days, expected_type=type_hints["devices_key_duration_days"])
            check_type(argname="argument https_enabled", value=https_enabled, expected_type=type_hints["https_enabled"])
            check_type(argname="argument network_flow_logging_on", value=network_flow_logging_on, expected_type=type_hints["network_flow_logging_on"])
            check_type(argname="argument posture_identity_collection_on", value=posture_identity_collection_on, expected_type=type_hints["posture_identity_collection_on"])
            check_type(argname="argument regional_routing_on", value=regional_routing_on, expected_type=type_hints["regional_routing_on"])
            check_type(argname="argument users_approval_on", value=users_approval_on, expected_type=type_hints["users_approval_on"])
            check_type(argname="argument users_role_allowed_to_join_external_tailnet", value=users_role_allowed_to_join_external_tailnet, expected_type=type_hints["users_role_allowed_to_join_external_tailnet"])
        self._values: typing.Dict[builtins.str, typing.Any] = {}
        if connection is not None:
            self._values["connection"] = connection
        if count is not None:
            self._values["count"] = count
        if depends_on is not None:
            self._values["depends_on"] = depends_on
        if for_each is not None:
            self._values["for_each"] = for_each
        if lifecycle is not None:
            self._values["lifecycle"] = lifecycle
        if provider is not None:
            self._values["provider"] = provider
        if provisioners is not None:
            self._values["provisioners"] = provisioners
        if acls_external_link is not None:
            self._values["acls_external_link"] = acls_external_link
        if acls_externally_managed_on is not None:
            self._values["acls_externally_managed_on"] = acls_externally_managed_on
        if devices_approval_on is not None:
            self._values["devices_approval_on"] = devices_approval_on
        if devices_auto_updates_on is not None:
            self._values["devices_auto_updates_on"] = devices_auto_updates_on
        if devices_key_duration_days is not None:
            self._values["devices_key_duration_days"] = devices_key_duration_days
        if https_enabled is not None:
            self._values["https_enabled"] = https_enabled
        if network_flow_logging_on is not None:
            self._values["network_flow_logging_on"] = network_flow_logging_on
        if posture_identity_collection_on is not None:
            self._values["posture_identity_collection_on"] = posture_identity_collection_on
        if regional_routing_on is not None:
            self._values["regional_routing_on"] = regional_routing_on
        if users_approval_on is not None:
            self._values["users_approval_on"] = users_approval_on
        if users_role_allowed_to_join_external_tailnet is not None:
            self._values["users_role_allowed_to_join_external_tailnet"] = users_role_allowed_to_join_external_tailnet

    @builtins.property
    def connection(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", "_cdktn_78ede62e.WinrmProvisionerConnection"]]:
        '''
        :stability: experimental
        '''
        result = self._values.get("connection")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", "_cdktn_78ede62e.WinrmProvisionerConnection"]], result)

    @builtins.property
    def count(
        self,
    ) -> typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]]:
        '''
        :stability: experimental
        '''
        result = self._values.get("count")
        return typing.cast(typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]], result)

    @builtins.property
    def depends_on(
        self,
    ) -> typing.Optional[typing.List["_cdktn_78ede62e.ITerraformDependable"]]:
        '''
        :stability: experimental
        '''
        result = self._values.get("depends_on")
        return typing.cast(typing.Optional[typing.List["_cdktn_78ede62e.ITerraformDependable"]], result)

    @builtins.property
    def for_each(self) -> typing.Optional["_cdktn_78ede62e.ITerraformIterator"]:
        '''
        :stability: experimental
        '''
        result = self._values.get("for_each")
        return typing.cast(typing.Optional["_cdktn_78ede62e.ITerraformIterator"], result)

    @builtins.property
    def lifecycle(
        self,
    ) -> typing.Optional["_cdktn_78ede62e.TerraformResourceLifecycle"]:
        '''
        :stability: experimental
        '''
        result = self._values.get("lifecycle")
        return typing.cast(typing.Optional["_cdktn_78ede62e.TerraformResourceLifecycle"], result)

    @builtins.property
    def provider(self) -> typing.Optional["_cdktn_78ede62e.TerraformProvider"]:
        '''
        :stability: experimental
        '''
        result = self._values.get("provider")
        return typing.cast(typing.Optional["_cdktn_78ede62e.TerraformProvider"], result)

    @builtins.property
    def provisioners(
        self,
    ) -> typing.Optional[typing.List[typing.Union["_cdktn_78ede62e.FileProvisioner", "_cdktn_78ede62e.LocalExecProvisioner", "_cdktn_78ede62e.RemoteExecProvisioner"]]]:
        '''
        :stability: experimental
        '''
        result = self._values.get("provisioners")
        return typing.cast(typing.Optional[typing.List[typing.Union["_cdktn_78ede62e.FileProvisioner", "_cdktn_78ede62e.LocalExecProvisioner", "_cdktn_78ede62e.RemoteExecProvisioner"]]], result)

    @builtins.property
    def acls_external_link(self) -> typing.Optional[builtins.str]:
        '''Link to your external ACL definition or management system. Must be a valid URL.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_external_link TailnetSettings#acls_external_link}
        '''
        result = self._values.get("acls_external_link")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def acls_externally_managed_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Prevent users from editing policies in the admin console to avoid conflicts with external management workflows like GitOps or Terraform.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#acls_externally_managed_on TailnetSettings#acls_externally_managed_on}
        '''
        result = self._values.get("acls_externally_managed_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def devices_approval_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether device approval is enabled for the tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_approval_on TailnetSettings#devices_approval_on}
        '''
        result = self._values.get("devices_approval_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def devices_auto_updates_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether auto updates are enabled for devices that belong to this tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_auto_updates_on TailnetSettings#devices_auto_updates_on}
        '''
        result = self._values.get("devices_auto_updates_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def devices_key_duration_days(self) -> typing.Optional[jsii.Number]:
        '''The key expiry duration for devices on this tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#devices_key_duration_days TailnetSettings#devices_key_duration_days}
        '''
        result = self._values.get("devices_key_duration_days")
        return typing.cast(typing.Optional[jsii.Number], result)

    @builtins.property
    def https_enabled(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether provisioning of HTTPS certificates is enabled for the tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#https_enabled TailnetSettings#https_enabled}
        '''
        result = self._values.get("https_enabled")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def network_flow_logging_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether network flow logs are enabled for the tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#network_flow_logging_on TailnetSettings#network_flow_logging_on}
        '''
        result = self._values.get("network_flow_logging_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def posture_identity_collection_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether identity collection is enabled for device posture integrations for the tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#posture_identity_collection_on TailnetSettings#posture_identity_collection_on}
        '''
        result = self._values.get("posture_identity_collection_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def regional_routing_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether regional routing is enabled for the tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#regional_routing_on TailnetSettings#regional_routing_on}
        '''
        result = self._values.get("regional_routing_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def users_approval_on(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether user approval is enabled for this tailnet.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_approval_on TailnetSettings#users_approval_on}
        '''
        result = self._values.get("users_approval_on")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def users_role_allowed_to_join_external_tailnet(
        self,
    ) -> typing.Optional[builtins.str]:
        '''Which user roles are allowed to join external tailnets.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_settings#users_role_allowed_to_join_external_tailnet TailnetSettings#users_role_allowed_to_join_external_tailnet}
        '''
        result = self._values.get("users_role_allowed_to_join_external_tailnet")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "TailnetSettingsConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "TailnetSettings",
    "TailnetSettingsConfig",
]

publication.publish()

def _typecheckingstub__dbb71315ce324f6371c806951765b9d063631af8ed2172a8f0f88ac19fb959b5(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    acls_external_link: typing.Optional[builtins.str] = None,
    acls_externally_managed_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_approval_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_auto_updates_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_key_duration_days: typing.Optional[jsii.Number] = None,
    https_enabled: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    network_flow_logging_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    posture_identity_collection_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    regional_routing_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    users_approval_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    users_role_allowed_to_join_external_tailnet: typing.Optional[builtins.str] = None,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__143fdbd3465f8148dbc3428de1e381d2e2f4a7d4cc8b3f01eb10534d1863e187(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__fea4259eda6f308c157fb2cc165a6bdea20fd1f2db432dff41117a62d1c30346(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__36325c359a7c2bacb4e836f7404d909ec0094d67deddad595990c0e230055bf1(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4e1dff6a145f0af1d71d9135a7c1901f80b9730e37ca6373456747e0d7bccb56(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__dc843244c6537d44a9bf10b569021c1316c99e43dde36620577d72b66b69bae3(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c323018adcd99f7b850070e73c9c1558e9ca9cb0212f925c5e802c58c1f0a2be(
    value: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__fa7a5c6d49efe9c57620033ec25814694e9628d9a80d86643fefbd08b21c1f32(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__346d4abfed08bcc5e3752edec17ee81f3aa89d3e7f46446ab4c13c9f307b757f(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__8fd8cef20312361ec73c6348e29fcb2b20a4df8c86a6e2feb5af3160e83ea17b(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__ec49d64a54d0aec5cbf765702033d3197e7dd7f9c19139ab57654c90ae51cb2e(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__5e54502c4a6e59eda180c23187c8181f3de1db5651b89156eb37fefc851cfef9(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__5552288a46b2d26de318d077d2fc16c8f26d7f5f595ef526409039354fd1cbeb(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__86c6dd20e39ed19251a083661ecb4fe2906d9fb7eab7c9c8f39fcfda282861b1(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    acls_external_link: typing.Optional[builtins.str] = None,
    acls_externally_managed_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_approval_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_auto_updates_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    devices_key_duration_days: typing.Optional[jsii.Number] = None,
    https_enabled: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    network_flow_logging_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    posture_identity_collection_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    regional_routing_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    users_approval_on: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    users_role_allowed_to_join_external_tailnet: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass
