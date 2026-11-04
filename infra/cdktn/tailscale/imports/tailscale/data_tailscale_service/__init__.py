r'''
# `data_tailscale_service`

Refer to the OpenTofu Registry for docs: [`data_tailscale_service`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service).
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


class DataTailscaleService(
    _cdktn_78ede62e.TerraformDataSource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleService.DataTailscaleService",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service tailscale_service}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        name: builtins.str,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service tailscale_service} Data Source.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param name: The name of the Service (e.g. ``svc:my-service``). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service#name DataTailscaleService#name}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__9ec72b4534538e33ae99e4fd903702a57a6b5501ac244980e59e6c718646fd39)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = DataTailscaleServiceConfig(
            name=name,
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
        '''Generates CDKTN code for importing a DataTailscaleService resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the DataTailscaleService to import.
        :param import_from_id: The id of the existing DataTailscaleService that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the DataTailscaleService to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__0bf07271460d2dbcd44b62b9930d25b59727b656a0f14b389bcf514f1dadd965)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

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
    @jsii.member(jsii_name="addrs")
    def addrs(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "addrs"))

    @builtins.property
    @jsii.member(jsii_name="comment")
    def comment(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "comment"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="ports")
    def ports(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "ports"))

    @builtins.property
    @jsii.member(jsii_name="tags")
    def tags(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "tags"))

    @builtins.property
    @jsii.member(jsii_name="nameInput")
    def name_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "nameInput"))

    @builtins.property
    @jsii.member(jsii_name="name")
    def name(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "name"))

    @name.setter
    def name(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f3793395231a9ef2e92b649dd3a6e85036b44a8a5cb34b49fd5630bc3601261b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "name", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dataTailscaleService.DataTailscaleServiceConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "name": "name",
    },
)
class DataTailscaleServiceConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        name: builtins.str,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param name: The name of the Service (e.g. ``svc:my-service``). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service#name DataTailscaleService#name}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c12b170d5c2b385bfa0f0f47ecd1b932b978fc0350fcb439de60a0f2d641ba65)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument name", value=name, expected_type=type_hints["name"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "name": name,
        }
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
    def name(self) -> builtins.str:
        '''The name of the Service (e.g. ``svc:my-service``).

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/service#name DataTailscaleService#name}
        '''
        result = self._values.get("name")
        assert result is not None, "Required property 'name' is missing"
        return typing.cast(builtins.str, result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DataTailscaleServiceConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "DataTailscaleService",
    "DataTailscaleServiceConfig",
]

publication.publish()

def _typecheckingstub__9ec72b4534538e33ae99e4fd903702a57a6b5501ac244980e59e6c718646fd39(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    name: builtins.str,
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

def _typecheckingstub__0bf07271460d2dbcd44b62b9930d25b59727b656a0f14b389bcf514f1dadd965(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f3793395231a9ef2e92b649dd3a6e85036b44a8a5cb34b49fd5630bc3601261b(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c12b170d5c2b385bfa0f0f47ecd1b932b978fc0350fcb439de60a0f2d641ba65(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    name: builtins.str,
) -> None:
    """Type checking stubs"""
    pass
