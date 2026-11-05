r'''
# `tailscale_dns_configuration`

Refer to the OpenTofu Registry for docs: [`tailscale_dns_configuration`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration).
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


class DnsConfiguration(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfiguration",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration tailscale_dns_configuration}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        magic_dns: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        nameservers: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationNameservers", typing.Dict[builtins.str, typing.Any]]]]] = None,
        override_local_dns: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        search_paths: typing.Optional[typing.Sequence[builtins.str]] = None,
        split_dns: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationSplitDns", typing.Dict[builtins.str, typing.Any]]]]] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration tailscale_dns_configuration} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param magic_dns: Whether or not to enable MagicDNS. Defaults to true. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#magic_dns DnsConfiguration#magic_dns}
        :param nameservers: nameservers block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#nameservers DnsConfiguration#nameservers}
        :param override_local_dns: When enabled, use the configured DNS servers in ``nameservers`` to resolve names outside the tailnet. When disabled, devices will prefer their local DNS configuration. Defaults to false. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#override_local_dns DnsConfiguration#override_local_dns}
        :param search_paths: Additional search domains. When MagicDNS is on, the tailnet domain is automatically included as the first search domain. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#search_paths DnsConfiguration#search_paths}
        :param split_dns: split_dns block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#split_dns DnsConfiguration#split_dns}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a5dc0a6426a2ec33064c6b81e57454c383fbb498a6067cbe78dc79a1a03d5064)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = DnsConfigurationConfig(
            magic_dns=magic_dns,
            nameservers=nameservers,
            override_local_dns=override_local_dns,
            search_paths=search_paths,
            split_dns=split_dns,
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
        '''Generates CDKTN code for importing a DnsConfiguration resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the DnsConfiguration to import.
        :param import_from_id: The id of the existing DnsConfiguration that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the DnsConfiguration to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__d155fb4118938f1ce51e9d229d6ff844a6d992db13dfd2fa00fddf7f435b6716)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="putNameservers")
    def put_nameservers(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationNameservers", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__49f6a397f40ebc2246149ee704af71f08883e6c8d544929a0c514a16c8005fe0)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putNameservers", [value]))

    @jsii.member(jsii_name="putSplitDns")
    def put_split_dns(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationSplitDns", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__72ad56658d4c534c6398df49303a6b64f9cec0434ced6c497a66cc8e45409c26)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putSplitDns", [value]))

    @jsii.member(jsii_name="resetMagicDns")
    def reset_magic_dns(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetMagicDns", []))

    @jsii.member(jsii_name="resetNameservers")
    def reset_nameservers(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetNameservers", []))

    @jsii.member(jsii_name="resetOverrideLocalDns")
    def reset_override_local_dns(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetOverrideLocalDns", []))

    @jsii.member(jsii_name="resetSearchPaths")
    def reset_search_paths(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetSearchPaths", []))

    @jsii.member(jsii_name="resetSplitDns")
    def reset_split_dns(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetSplitDns", []))

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
    @jsii.member(jsii_name="nameservers")
    def nameservers(self) -> "DnsConfigurationNameserversList":
        return typing.cast("DnsConfigurationNameserversList", jsii.get(self, "nameservers"))

    @builtins.property
    @jsii.member(jsii_name="splitDns")
    def split_dns(self) -> "DnsConfigurationSplitDnsList":
        return typing.cast("DnsConfigurationSplitDnsList", jsii.get(self, "splitDns"))

    @builtins.property
    @jsii.member(jsii_name="magicDnsInput")
    def magic_dns_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "magicDnsInput"))

    @builtins.property
    @jsii.member(jsii_name="nameserversInput")
    def nameservers_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]], jsii.get(self, "nameserversInput"))

    @builtins.property
    @jsii.member(jsii_name="overrideLocalDnsInput")
    def override_local_dns_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "overrideLocalDnsInput"))

    @builtins.property
    @jsii.member(jsii_name="searchPathsInput")
    def search_paths_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "searchPathsInput"))

    @builtins.property
    @jsii.member(jsii_name="splitDnsInput")
    def split_dns_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]], jsii.get(self, "splitDnsInput"))

    @builtins.property
    @jsii.member(jsii_name="magicDns")
    def magic_dns(self) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "magicDns"))

    @magic_dns.setter
    def magic_dns(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__9340af7b1df87e8657c6750f0e3dfd2f34d3259aa90df415ca0f9f9b48305f00)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "magicDns", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="overrideLocalDns")
    def override_local_dns(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "overrideLocalDns"))

    @override_local_dns.setter
    def override_local_dns(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__3754d7a37cfb2b187b41c2251c6ef9fe16a756b9b14f05e80e93366a557b9936)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "overrideLocalDns", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="searchPaths")
    def search_paths(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "searchPaths"))

    @search_paths.setter
    def search_paths(self, value: typing.List[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__27827581d455d354fdeba9a49d56b9cbcf478bc6f9e174da7dee1a7cf191d329)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "searchPaths", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "magic_dns": "magicDns",
        "nameservers": "nameservers",
        "override_local_dns": "overrideLocalDns",
        "search_paths": "searchPaths",
        "split_dns": "splitDns",
    },
)
class DnsConfigurationConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        magic_dns: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        nameservers: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationNameservers", typing.Dict[builtins.str, typing.Any]]]]] = None,
        override_local_dns: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        search_paths: typing.Optional[typing.Sequence[builtins.str]] = None,
        split_dns: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationSplitDns", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param magic_dns: Whether or not to enable MagicDNS. Defaults to true. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#magic_dns DnsConfiguration#magic_dns}
        :param nameservers: nameservers block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#nameservers DnsConfiguration#nameservers}
        :param override_local_dns: When enabled, use the configured DNS servers in ``nameservers`` to resolve names outside the tailnet. When disabled, devices will prefer their local DNS configuration. Defaults to false. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#override_local_dns DnsConfiguration#override_local_dns}
        :param search_paths: Additional search domains. When MagicDNS is on, the tailnet domain is automatically included as the first search domain. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#search_paths DnsConfiguration#search_paths}
        :param split_dns: split_dns block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#split_dns DnsConfiguration#split_dns}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7588a8456f13fc35d13a4a3168d7e752da58da19db6f0867e580bbc5d292101d)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument magic_dns", value=magic_dns, expected_type=type_hints["magic_dns"])
            check_type(argname="argument nameservers", value=nameservers, expected_type=type_hints["nameservers"])
            check_type(argname="argument override_local_dns", value=override_local_dns, expected_type=type_hints["override_local_dns"])
            check_type(argname="argument search_paths", value=search_paths, expected_type=type_hints["search_paths"])
            check_type(argname="argument split_dns", value=split_dns, expected_type=type_hints["split_dns"])
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
        if magic_dns is not None:
            self._values["magic_dns"] = magic_dns
        if nameservers is not None:
            self._values["nameservers"] = nameservers
        if override_local_dns is not None:
            self._values["override_local_dns"] = override_local_dns
        if search_paths is not None:
            self._values["search_paths"] = search_paths
        if split_dns is not None:
            self._values["split_dns"] = split_dns

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
    def magic_dns(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Whether or not to enable MagicDNS. Defaults to true.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#magic_dns DnsConfiguration#magic_dns}
        '''
        result = self._values.get("magic_dns")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def nameservers(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]]:
        '''nameservers block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#nameservers DnsConfiguration#nameservers}
        '''
        result = self._values.get("nameservers")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]], result)

    @builtins.property
    def override_local_dns(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''When enabled, use the configured DNS servers in ``nameservers`` to resolve names outside the tailnet.

        When disabled, devices will prefer their local DNS configuration. Defaults to false.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#override_local_dns DnsConfiguration#override_local_dns}
        '''
        result = self._values.get("override_local_dns")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def search_paths(self) -> typing.Optional[typing.List[builtins.str]]:
        '''Additional search domains. When MagicDNS is on, the tailnet domain is automatically included as the first search domain.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#search_paths DnsConfiguration#search_paths}
        '''
        result = self._values.get("search_paths")
        return typing.cast(typing.Optional[typing.List[builtins.str]], result)

    @builtins.property
    def split_dns(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]]:
        '''split_dns block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#split_dns DnsConfiguration#split_dns}
        '''
        result = self._values.get("split_dns")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DnsConfigurationConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


@jsii.data_type(
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationNameservers",
    jsii_struct_bases=[],
    name_mapping={"address": "address", "use_with_exit_node": "useWithExitNode"},
)
class DnsConfigurationNameservers:
    def __init__(
        self,
        *,
        address: builtins.str,
        use_with_exit_node: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
    ) -> None:
        '''
        :param address: The nameserver's IPv4 or IPv6 address. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#address DnsConfiguration#address}
        :param use_with_exit_node: This nameserver will continue to be used when an exit node is selected (requires Tailscale v1.88.1 or later). Defaults to false. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#use_with_exit_node DnsConfiguration#use_with_exit_node}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c72cd52f5e935dde9a3d47d6f37776bd69357c94d6baf3c184dd9913bab39ffe)
            check_type(argname="argument address", value=address, expected_type=type_hints["address"])
            check_type(argname="argument use_with_exit_node", value=use_with_exit_node, expected_type=type_hints["use_with_exit_node"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "address": address,
        }
        if use_with_exit_node is not None:
            self._values["use_with_exit_node"] = use_with_exit_node

    @builtins.property
    def address(self) -> builtins.str:
        '''The nameserver's IPv4 or IPv6 address.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#address DnsConfiguration#address}
        '''
        result = self._values.get("address")
        assert result is not None, "Required property 'address' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def use_with_exit_node(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''This nameserver will continue to be used when an exit node is selected (requires Tailscale v1.88.1 or later). Defaults to false.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#use_with_exit_node DnsConfiguration#use_with_exit_node}
        '''
        result = self._values.get("use_with_exit_node")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DnsConfigurationNameservers(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class DnsConfigurationNameserversList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationNameserversList",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        wraps_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param wraps_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f58023f2f0ad187c09cf4190f36cd194976d6d55281c383be3b315c29c0f0d52)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "DnsConfigurationNameserversOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__e8650c90883c8241ed00208663b31ed1958cf033bb45fc79ba98c61a51e47864)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("DnsConfigurationNameserversOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationNameservers"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a7d214d4243c560fd12049dc79f7d5be1824e7c92f257a3801abd49dbec65c0e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class DnsConfigurationNameserversOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationNameserversOutputReference",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        complex_object_index: jsii.Number,
        complex_object_is_from_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param complex_object_index: the index of this item in the list.
        :param complex_object_is_from_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__cad6d1ff1a1db168feb2f5d926351fc6b9b8999cb9d317c975514a6ae74c10e9)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @jsii.member(jsii_name="resetUseWithExitNode")
    def reset_use_with_exit_node(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUseWithExitNode", []))

    @builtins.property
    @jsii.member(jsii_name="addressInput")
    def address_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "addressInput"))

    @builtins.property
    @jsii.member(jsii_name="useWithExitNodeInput")
    def use_with_exit_node_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "useWithExitNodeInput"))

    @builtins.property
    @jsii.member(jsii_name="address")
    def address(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "address"))

    @address.setter
    def address(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f696a4b00958eda2963ac212aa1e484acd5f9f1697c4977d025a5b271282a0b1)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "address", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="useWithExitNode")
    def use_with_exit_node(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "useWithExitNode"))

    @use_with_exit_node.setter
    def use_with_exit_node(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__cb0e2fe66cc0808de6535914c7dcda2d66199fa8fabd939d919b3feaf860c933)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "useWithExitNode", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationNameservers"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationNameservers"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationNameservers"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__90d143802e08a7708079f5344623ee21ddfce33aa7b60b5d1e05ddcc2afdfe49)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDns",
    jsii_struct_bases=[],
    name_mapping={"domain": "domain", "nameservers": "nameservers"},
)
class DnsConfigurationSplitDns:
    def __init__(
        self,
        *,
        domain: builtins.str,
        nameservers: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationSplitDnsNameservers", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''
        :param domain: The nameservers will be used only for this domain. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#domain DnsConfiguration#domain}
        :param nameservers: nameservers block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#nameservers DnsConfiguration#nameservers}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__20f10a1afb606127669eace77d882afdf6c62db558c509f7ee6eb6c51cb74390)
            check_type(argname="argument domain", value=domain, expected_type=type_hints["domain"])
            check_type(argname="argument nameservers", value=nameservers, expected_type=type_hints["nameservers"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "domain": domain,
        }
        if nameservers is not None:
            self._values["nameservers"] = nameservers

    @builtins.property
    def domain(self) -> builtins.str:
        '''The nameservers will be used only for this domain.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#domain DnsConfiguration#domain}
        '''
        result = self._values.get("domain")
        assert result is not None, "Required property 'domain' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def nameservers(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]]:
        '''nameservers block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#nameservers DnsConfiguration#nameservers}
        '''
        result = self._values.get("nameservers")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DnsConfigurationSplitDns(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class DnsConfigurationSplitDnsList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDnsList",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        wraps_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param wraps_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__3baa6613b1ddee5830ff682d112a77f50aad5aeaff859c0848e15d3bc3739fd9)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "DnsConfigurationSplitDnsOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__22c84508d1c8a0ecc4bec842f100bb1934c97229f621d59e5c55e5724a4b772f)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("DnsConfigurationSplitDnsOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDns"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7db1858b8949dc3dabc5e3c2ff4c37b3ba74d36514c6bf0648b2380c3a041f7f)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDnsNameservers",
    jsii_struct_bases=[],
    name_mapping={"address": "address", "use_with_exit_node": "useWithExitNode"},
)
class DnsConfigurationSplitDnsNameservers:
    def __init__(
        self,
        *,
        address: builtins.str,
        use_with_exit_node: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
    ) -> None:
        '''
        :param address: The nameserver's IPv4 or IPv6 address. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#address DnsConfiguration#address}
        :param use_with_exit_node: This nameserver will continue to be used when an exit node is selected (requires Tailscale v1.88.1 or later). Defaults to false. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#use_with_exit_node DnsConfiguration#use_with_exit_node}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__105e1f87d751e9c105fe99eb61f346956aeceef97a4cbb1ebf89fdd8cbf89bf4)
            check_type(argname="argument address", value=address, expected_type=type_hints["address"])
            check_type(argname="argument use_with_exit_node", value=use_with_exit_node, expected_type=type_hints["use_with_exit_node"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "address": address,
        }
        if use_with_exit_node is not None:
            self._values["use_with_exit_node"] = use_with_exit_node

    @builtins.property
    def address(self) -> builtins.str:
        '''The nameserver's IPv4 or IPv6 address.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#address DnsConfiguration#address}
        '''
        result = self._values.get("address")
        assert result is not None, "Required property 'address' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def use_with_exit_node(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''This nameserver will continue to be used when an exit node is selected (requires Tailscale v1.88.1 or later). Defaults to false.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/dns_configuration#use_with_exit_node DnsConfiguration#use_with_exit_node}
        '''
        result = self._values.get("use_with_exit_node")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DnsConfigurationSplitDnsNameservers(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class DnsConfigurationSplitDnsNameserversList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDnsNameserversList",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        wraps_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param wraps_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__33e74742124309237f929b591db07340fd3d93c5aaa1556a952d5cf4ae616ce1)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(
        self,
        index: jsii.Number,
    ) -> "DnsConfigurationSplitDnsNameserversOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__2d61548cd606b6e0e0b02c4a1efcc67006724f98ed9931f27f90d8337c038cda)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("DnsConfigurationSplitDnsNameserversOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c812e26834c41355bb4cc1e1a8a91c6eb1862d4681a0ebb1d3cff7799485fb38)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class DnsConfigurationSplitDnsNameserversOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDnsNameserversOutputReference",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        complex_object_index: jsii.Number,
        complex_object_is_from_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param complex_object_index: the index of this item in the list.
        :param complex_object_is_from_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__65acebf86e8ec9c0007b0b476c91b80a64574aa27edf151103ad87452fbe2499)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @jsii.member(jsii_name="resetUseWithExitNode")
    def reset_use_with_exit_node(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUseWithExitNode", []))

    @builtins.property
    @jsii.member(jsii_name="addressInput")
    def address_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "addressInput"))

    @builtins.property
    @jsii.member(jsii_name="useWithExitNodeInput")
    def use_with_exit_node_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "useWithExitNodeInput"))

    @builtins.property
    @jsii.member(jsii_name="address")
    def address(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "address"))

    @address.setter
    def address(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__5c822c4f7c0f76145eaf96ab47f7d0971ae0abc36b51ec05484267e1d65a90a0)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "address", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="useWithExitNode")
    def use_with_exit_node(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "useWithExitNode"))

    @use_with_exit_node.setter
    def use_with_exit_node(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__119158111139b235cc63869eca2b4e0bbeb8e61b9fb540bc90f3d4e10d0e80de)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "useWithExitNode", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDnsNameservers"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDnsNameservers"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDnsNameservers"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__6df029fc5e738604bc552276bc9bdda75723bcac941a3b82bfba235d4428b34f)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class DnsConfigurationSplitDnsOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dnsConfiguration.DnsConfigurationSplitDnsOutputReference",
):
    def __init__(
        self,
        terraform_resource: "_cdktn_78ede62e.IInterpolatingParent",
        terraform_attribute: builtins.str,
        complex_object_index: jsii.Number,
        complex_object_is_from_set: builtins.bool,
    ) -> None:
        '''
        :param terraform_resource: The parent resource.
        :param terraform_attribute: The attribute on the parent resource this class is referencing.
        :param complex_object_index: the index of this item in the list.
        :param complex_object_is_from_set: whether the list is wrapping a set (will add tolist() to be able to access an item via an index).
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__087859c80fb512d00f060decdcab3ef70a67381e18c830e32aff69bc1e9461d5)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @jsii.member(jsii_name="putNameservers")
    def put_nameservers(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DnsConfigurationSplitDnsNameservers", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__aa96c3b2602ae423cdfa7ec19c19e6f0659fc310c255e1a631c2efd7efc0e48b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putNameservers", [value]))

    @jsii.member(jsii_name="resetNameservers")
    def reset_nameservers(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetNameservers", []))

    @builtins.property
    @jsii.member(jsii_name="nameservers")
    def nameservers(self) -> "DnsConfigurationSplitDnsNameserversList":
        return typing.cast("DnsConfigurationSplitDnsNameserversList", jsii.get(self, "nameservers"))

    @builtins.property
    @jsii.member(jsii_name="domainInput")
    def domain_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "domainInput"))

    @builtins.property
    @jsii.member(jsii_name="nameserversInput")
    def nameservers_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DnsConfigurationSplitDnsNameservers"]]], jsii.get(self, "nameserversInput"))

    @builtins.property
    @jsii.member(jsii_name="domain")
    def domain(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "domain"))

    @domain.setter
    def domain(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__77fd4f8d15ce4366ad2cf2d6189593b6327efcb4fb8e71a1488df9d967a4eda2)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "domain", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDns"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDns"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DnsConfigurationSplitDns"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__6d7255387f924451cf7e8d5356c0a342de9da2d40aacbd80ea1b3e105b780dcc)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


__all__ = [
    "DnsConfiguration",
    "DnsConfigurationConfig",
    "DnsConfigurationNameservers",
    "DnsConfigurationNameserversList",
    "DnsConfigurationNameserversOutputReference",
    "DnsConfigurationSplitDns",
    "DnsConfigurationSplitDnsList",
    "DnsConfigurationSplitDnsNameservers",
    "DnsConfigurationSplitDnsNameserversList",
    "DnsConfigurationSplitDnsNameserversOutputReference",
    "DnsConfigurationSplitDnsOutputReference",
]

publication.publish()

def _typecheckingstub__a5dc0a6426a2ec33064c6b81e57454c383fbb498a6067cbe78dc79a1a03d5064(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    magic_dns: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    nameservers: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationNameservers, typing.Dict[builtins.str, typing.Any]]]]] = None,
    override_local_dns: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    search_paths: typing.Optional[typing.Sequence[builtins.str]] = None,
    split_dns: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationSplitDns, typing.Dict[builtins.str, typing.Any]]]]] = None,
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

def _typecheckingstub__d155fb4118938f1ce51e9d229d6ff844a6d992db13dfd2fa00fddf7f435b6716(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__49f6a397f40ebc2246149ee704af71f08883e6c8d544929a0c514a16c8005fe0(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationNameservers, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__72ad56658d4c534c6398df49303a6b64f9cec0434ced6c497a66cc8e45409c26(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationSplitDns, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__9340af7b1df87e8657c6750f0e3dfd2f34d3259aa90df415ca0f9f9b48305f00(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__3754d7a37cfb2b187b41c2251c6ef9fe16a756b9b14f05e80e93366a557b9936(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__27827581d455d354fdeba9a49d56b9cbcf478bc6f9e174da7dee1a7cf191d329(
    value: typing.List[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7588a8456f13fc35d13a4a3168d7e752da58da19db6f0867e580bbc5d292101d(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    magic_dns: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    nameservers: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationNameservers, typing.Dict[builtins.str, typing.Any]]]]] = None,
    override_local_dns: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    search_paths: typing.Optional[typing.Sequence[builtins.str]] = None,
    split_dns: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationSplitDns, typing.Dict[builtins.str, typing.Any]]]]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c72cd52f5e935dde9a3d47d6f37776bd69357c94d6baf3c184dd9913bab39ffe(
    *,
    address: builtins.str,
    use_with_exit_node: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f58023f2f0ad187c09cf4190f36cd194976d6d55281c383be3b315c29c0f0d52(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__e8650c90883c8241ed00208663b31ed1958cf033bb45fc79ba98c61a51e47864(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__a7d214d4243c560fd12049dc79f7d5be1824e7c92f257a3801abd49dbec65c0e(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[DnsConfigurationNameservers]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__cad6d1ff1a1db168feb2f5d926351fc6b9b8999cb9d317c975514a6ae74c10e9(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f696a4b00958eda2963ac212aa1e484acd5f9f1697c4977d025a5b271282a0b1(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__cb0e2fe66cc0808de6535914c7dcda2d66199fa8fabd939d919b3feaf860c933(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__90d143802e08a7708079f5344623ee21ddfce33aa7b60b5d1e05ddcc2afdfe49(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, DnsConfigurationNameservers]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__20f10a1afb606127669eace77d882afdf6c62db558c509f7ee6eb6c51cb74390(
    *,
    domain: builtins.str,
    nameservers: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationSplitDnsNameservers, typing.Dict[builtins.str, typing.Any]]]]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__3baa6613b1ddee5830ff682d112a77f50aad5aeaff859c0848e15d3bc3739fd9(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__22c84508d1c8a0ecc4bec842f100bb1934c97229f621d59e5c55e5724a4b772f(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7db1858b8949dc3dabc5e3c2ff4c37b3ba74d36514c6bf0648b2380c3a041f7f(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[DnsConfigurationSplitDns]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__105e1f87d751e9c105fe99eb61f346956aeceef97a4cbb1ebf89fdd8cbf89bf4(
    *,
    address: builtins.str,
    use_with_exit_node: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__33e74742124309237f929b591db07340fd3d93c5aaa1556a952d5cf4ae616ce1(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__2d61548cd606b6e0e0b02c4a1efcc67006724f98ed9931f27f90d8337c038cda(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c812e26834c41355bb4cc1e1a8a91c6eb1862d4681a0ebb1d3cff7799485fb38(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[DnsConfigurationSplitDnsNameservers]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__65acebf86e8ec9c0007b0b476c91b80a64574aa27edf151103ad87452fbe2499(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__5c822c4f7c0f76145eaf96ab47f7d0971ae0abc36b51ec05484267e1d65a90a0(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__119158111139b235cc63869eca2b4e0bbeb8e61b9fb540bc90f3d4e10d0e80de(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__6df029fc5e738604bc552276bc9bdda75723bcac941a3b82bfba235d4428b34f(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, DnsConfigurationSplitDnsNameservers]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__087859c80fb512d00f060decdcab3ef70a67381e18c830e32aff69bc1e9461d5(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__aa96c3b2602ae423cdfa7ec19c19e6f0659fc310c255e1a631c2efd7efc0e48b(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DnsConfigurationSplitDnsNameservers, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__77fd4f8d15ce4366ad2cf2d6189593b6327efcb4fb8e71a1488df9d967a4eda2(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__6d7255387f924451cf7e8d5356c0a342de9da2d40aacbd80ea1b3e105b780dcc(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, DnsConfigurationSplitDns]],
) -> None:
    """Type checking stubs"""
    pass
