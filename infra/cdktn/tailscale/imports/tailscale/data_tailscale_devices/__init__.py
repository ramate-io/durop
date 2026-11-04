r'''
# `data_tailscale_devices`

Refer to the OpenTofu Registry for docs: [`data_tailscale_devices`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices).
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


class DataTailscaleDevices(
    _cdktn_78ede62e.TerraformDataSource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevices",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices tailscale_devices}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        devices: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesDevices", typing.Dict[builtins.str, typing.Any]]]]] = None,
        filter: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesFilter", typing.Dict[builtins.str, typing.Any]]]]] = None,
        name_prefix: typing.Optional[builtins.str] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices tailscale_devices} Data Source.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param devices: devices block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#devices DataTailscaleDevices#devices}
        :param filter: filter block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#filter DataTailscaleDevices#filter}
        :param name_prefix: Filters the device list to elements whose name has the provided prefix. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#name_prefix DataTailscaleDevices#name_prefix}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__11a60df653910ef53f47cc4a6658c6406c75256f5785eead7450d71bc83ebbae)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = DataTailscaleDevicesConfig(
            devices=devices,
            filter=filter,
            name_prefix=name_prefix,
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
        '''Generates CDKTN code for importing a DataTailscaleDevices resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the DataTailscaleDevices to import.
        :param import_from_id: The id of the existing DataTailscaleDevices that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the DataTailscaleDevices to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__9d099ae89ff091da8abb0d95584e35407ec0dabe882fda6aa0c178496aa0eb2a)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="putDevices")
    def put_devices(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesDevices", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__b3753f3eef40efc615864eaef0fce1dffd05d91914be625f03f7f5c4046f5467)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putDevices", [value]))

    @jsii.member(jsii_name="putFilter")
    def put_filter(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesFilter", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__6da875a20fe7b0f61bd8bfe1a6ee69ee30961589e81c5bdd0a4224de946e52fd)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putFilter", [value]))

    @jsii.member(jsii_name="resetDevices")
    def reset_devices(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetDevices", []))

    @jsii.member(jsii_name="resetFilter")
    def reset_filter(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetFilter", []))

    @jsii.member(jsii_name="resetNamePrefix")
    def reset_name_prefix(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetNamePrefix", []))

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
    @jsii.member(jsii_name="devices")
    def devices(self) -> "DataTailscaleDevicesDevicesList":
        return typing.cast("DataTailscaleDevicesDevicesList", jsii.get(self, "devices"))

    @builtins.property
    @jsii.member(jsii_name="filter")
    def filter(self) -> "DataTailscaleDevicesFilterList":
        return typing.cast("DataTailscaleDevicesFilterList", jsii.get(self, "filter"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="devicesInput")
    def devices_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]], jsii.get(self, "devicesInput"))

    @builtins.property
    @jsii.member(jsii_name="filterInput")
    def filter_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]], jsii.get(self, "filterInput"))

    @builtins.property
    @jsii.member(jsii_name="namePrefixInput")
    def name_prefix_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "namePrefixInput"))

    @builtins.property
    @jsii.member(jsii_name="namePrefix")
    def name_prefix(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "namePrefix"))

    @name_prefix.setter
    def name_prefix(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__529f3badfe511469c68672d03f261f4e6dba7a3a21e4cb979e96622e57dcf774)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "namePrefix", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "devices": "devices",
        "filter": "filter",
        "name_prefix": "namePrefix",
    },
)
class DataTailscaleDevicesConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        devices: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesDevices", typing.Dict[builtins.str, typing.Any]]]]] = None,
        filter: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["DataTailscaleDevicesFilter", typing.Dict[builtins.str, typing.Any]]]]] = None,
        name_prefix: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param devices: devices block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#devices DataTailscaleDevices#devices}
        :param filter: filter block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#filter DataTailscaleDevices#filter}
        :param name_prefix: Filters the device list to elements whose name has the provided prefix. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#name_prefix DataTailscaleDevices#name_prefix}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__85420799a0a896ea52545d9552d09d739ad7605a11cb8a1a0c3486b46e1fc68f)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument devices", value=devices, expected_type=type_hints["devices"])
            check_type(argname="argument filter", value=filter, expected_type=type_hints["filter"])
            check_type(argname="argument name_prefix", value=name_prefix, expected_type=type_hints["name_prefix"])
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
        if devices is not None:
            self._values["devices"] = devices
        if filter is not None:
            self._values["filter"] = filter
        if name_prefix is not None:
            self._values["name_prefix"] = name_prefix

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
    def devices(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]]:
        '''devices block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#devices DataTailscaleDevices#devices}
        '''
        result = self._values.get("devices")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]], result)

    @builtins.property
    def filter(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]]:
        '''filter block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#filter DataTailscaleDevices#filter}
        '''
        result = self._values.get("filter")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]], result)

    @builtins.property
    def name_prefix(self) -> typing.Optional[builtins.str]:
        '''Filters the device list to elements whose name has the provided prefix.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#name_prefix DataTailscaleDevices#name_prefix}
        '''
        result = self._values.get("name_prefix")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DataTailscaleDevicesConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


@jsii.data_type(
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesDevices",
    jsii_struct_bases=[],
    name_mapping={},
)
class DataTailscaleDevicesDevices:
    def __init__(self) -> None:
        self._values: typing.Dict[builtins.str, typing.Any] = {}

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DataTailscaleDevicesDevices(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class DataTailscaleDevicesDevicesList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesDevicesList",
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
            type_hints = cached_type_hints(_typecheckingstub__36500bcd8b8cf30cc1b3cc6cb490e79212663c3a643e60831c4c4740e7dc65cb)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "DataTailscaleDevicesDevicesOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__cd2dcb06478a68c840919f83f3383b810878104d375f01257acc2310fb136253)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("DataTailscaleDevicesDevicesOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesDevices"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4e66078272fe8b04792cd0cd44afa33bac5e907d74fdf6e7fb3833b196ee73d1)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class DataTailscaleDevicesDevicesOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesDevicesOutputReference",
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
            type_hints = cached_type_hints(_typecheckingstub__86c185e017db2b4addde44b0d3fc6dce49ccd8890116f903d3bf8c641ca9a51c)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @builtins.property
    @jsii.member(jsii_name="addresses")
    def addresses(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "addresses"))

    @builtins.property
    @jsii.member(jsii_name="authorized")
    def authorized(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "authorized"))

    @builtins.property
    @jsii.member(jsii_name="blocksIncomingConnections")
    def blocks_incoming_connections(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "blocksIncomingConnections"))

    @builtins.property
    @jsii.member(jsii_name="clientVersion")
    def client_version(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "clientVersion"))

    @builtins.property
    @jsii.member(jsii_name="created")
    def created(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "created"))

    @builtins.property
    @jsii.member(jsii_name="expires")
    def expires(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "expires"))

    @builtins.property
    @jsii.member(jsii_name="hostname")
    def hostname(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "hostname"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="isExternal")
    def is_external(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "isExternal"))

    @builtins.property
    @jsii.member(jsii_name="keyExpiryDisabled")
    def key_expiry_disabled(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "keyExpiryDisabled"))

    @builtins.property
    @jsii.member(jsii_name="lastSeen")
    def last_seen(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "lastSeen"))

    @builtins.property
    @jsii.member(jsii_name="machineKey")
    def machine_key(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "machineKey"))

    @builtins.property
    @jsii.member(jsii_name="name")
    def name(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "name"))

    @builtins.property
    @jsii.member(jsii_name="nodeId")
    def node_id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "nodeId"))

    @builtins.property
    @jsii.member(jsii_name="nodeKey")
    def node_key(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "nodeKey"))

    @builtins.property
    @jsii.member(jsii_name="os")
    def os(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "os"))

    @builtins.property
    @jsii.member(jsii_name="tags")
    def tags(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "tags"))

    @builtins.property
    @jsii.member(jsii_name="tailnetLockError")
    def tailnet_lock_error(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "tailnetLockError"))

    @builtins.property
    @jsii.member(jsii_name="tailnetLockKey")
    def tailnet_lock_key(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "tailnetLockKey"))

    @builtins.property
    @jsii.member(jsii_name="updateAvailable")
    def update_available(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "updateAvailable"))

    @builtins.property
    @jsii.member(jsii_name="user")
    def user(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "user"))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesDevices"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesDevices"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesDevices"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f9249efe7fffb0053ccf730db3e4dbd2de9f583c92a1d3b51ce3ec217c2c7b29)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesFilter",
    jsii_struct_bases=[],
    name_mapping={"name": "name", "values": "values"},
)
class DataTailscaleDevicesFilter:
    def __init__(
        self,
        *,
        name: builtins.str,
        values: typing.Sequence[builtins.str],
    ) -> None:
        '''
        :param name: The name must be a top-level device property, e.g. isEphemeral, tags, hostname, etc. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#name DataTailscaleDevices#name}
        :param values: The list of values to filter for. Values are matched as exact matches. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#values DataTailscaleDevices#values}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a3612994982358c517345a31817566b74ee5db342b153dc2bed84f2592950096)
            check_type(argname="argument name", value=name, expected_type=type_hints["name"])
            check_type(argname="argument values", value=values, expected_type=type_hints["values"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "name": name,
            "values": values,
        }

    @builtins.property
    def name(self) -> builtins.str:
        '''The name must be a top-level device property, e.g. isEphemeral, tags, hostname, etc.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#name DataTailscaleDevices#name}
        '''
        result = self._values.get("name")
        assert result is not None, "Required property 'name' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def values(self) -> typing.List[builtins.str]:
        '''The list of values to filter for. Values are matched as exact matches.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/datasources/devices#values DataTailscaleDevices#values}
        '''
        result = self._values.get("values")
        assert result is not None, "Required property 'values' is missing"
        return typing.cast(typing.List[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "DataTailscaleDevicesFilter(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class DataTailscaleDevicesFilterList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesFilterList",
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
            type_hints = cached_type_hints(_typecheckingstub__d5c3caae9b19758cc15ea1b5ffce0d96d08496c1d46a181dba8dfa61f5bda61b)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "DataTailscaleDevicesFilterOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__d07cb5edb77dc9efea9d7eafdfb8c28d04d2da3fac25d809b506aa7621774714)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("DataTailscaleDevicesFilterOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["DataTailscaleDevicesFilter"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__1d09ed8de503dff2dcbde325a6e270187ea212f7a015b206524e585ab70a7c97)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class DataTailscaleDevicesFilterOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.dataTailscaleDevices.DataTailscaleDevicesFilterOutputReference",
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
            type_hints = cached_type_hints(_typecheckingstub__116e674069f00db5676c35d223f7b6fff3b0c7f50d3432b6e1e9c89683f10cc3)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @builtins.property
    @jsii.member(jsii_name="nameInput")
    def name_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "nameInput"))

    @builtins.property
    @jsii.member(jsii_name="valuesInput")
    def values_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "valuesInput"))

    @builtins.property
    @jsii.member(jsii_name="name")
    def name(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "name"))

    @name.setter
    def name(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__145e0fa5eec77efc12006955ad6702494a92fc0b3b9482b25bb8d48d6bc7949e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "name", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="values")
    def values(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "values"))

    @values.setter
    def values(self, value: typing.List[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c7974307f11eb12d598a899685056c4850b1853e0d5a151c017505a2d80e5a96)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "values", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesFilter"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesFilter"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "DataTailscaleDevicesFilter"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__efc4edd7a9609a122263aded40cd4f4d0868d1199938d86dd5236e67a68d5f92)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


__all__ = [
    "DataTailscaleDevices",
    "DataTailscaleDevicesConfig",
    "DataTailscaleDevicesDevices",
    "DataTailscaleDevicesDevicesList",
    "DataTailscaleDevicesDevicesOutputReference",
    "DataTailscaleDevicesFilter",
    "DataTailscaleDevicesFilterList",
    "DataTailscaleDevicesFilterOutputReference",
]

publication.publish()

def _typecheckingstub__11a60df653910ef53f47cc4a6658c6406c75256f5785eead7450d71bc83ebbae(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    devices: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesDevices, typing.Dict[builtins.str, typing.Any]]]]] = None,
    filter: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesFilter, typing.Dict[builtins.str, typing.Any]]]]] = None,
    name_prefix: typing.Optional[builtins.str] = None,
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

def _typecheckingstub__9d099ae89ff091da8abb0d95584e35407ec0dabe882fda6aa0c178496aa0eb2a(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__b3753f3eef40efc615864eaef0fce1dffd05d91914be625f03f7f5c4046f5467(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesDevices, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__6da875a20fe7b0f61bd8bfe1a6ee69ee30961589e81c5bdd0a4224de946e52fd(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesFilter, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__529f3badfe511469c68672d03f261f4e6dba7a3a21e4cb979e96622e57dcf774(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__85420799a0a896ea52545d9552d09d739ad7605a11cb8a1a0c3486b46e1fc68f(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    devices: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesDevices, typing.Dict[builtins.str, typing.Any]]]]] = None,
    filter: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[DataTailscaleDevicesFilter, typing.Dict[builtins.str, typing.Any]]]]] = None,
    name_prefix: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__36500bcd8b8cf30cc1b3cc6cb490e79212663c3a643e60831c4c4740e7dc65cb(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__cd2dcb06478a68c840919f83f3383b810878104d375f01257acc2310fb136253(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4e66078272fe8b04792cd0cd44afa33bac5e907d74fdf6e7fb3833b196ee73d1(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[DataTailscaleDevicesDevices]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__86c185e017db2b4addde44b0d3fc6dce49ccd8890116f903d3bf8c641ca9a51c(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f9249efe7fffb0053ccf730db3e4dbd2de9f583c92a1d3b51ce3ec217c2c7b29(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, DataTailscaleDevicesDevices]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__a3612994982358c517345a31817566b74ee5db342b153dc2bed84f2592950096(
    *,
    name: builtins.str,
    values: typing.Sequence[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__d5c3caae9b19758cc15ea1b5ffce0d96d08496c1d46a181dba8dfa61f5bda61b(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__d07cb5edb77dc9efea9d7eafdfb8c28d04d2da3fac25d809b506aa7621774714(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__1d09ed8de503dff2dcbde325a6e270187ea212f7a015b206524e585ab70a7c97(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[DataTailscaleDevicesFilter]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__116e674069f00db5676c35d223f7b6fff3b0c7f50d3432b6e1e9c89683f10cc3(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__145e0fa5eec77efc12006955ad6702494a92fc0b3b9482b25bb8d48d6bc7949e(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c7974307f11eb12d598a899685056c4850b1853e0d5a151c017505a2d80e5a96(
    value: typing.List[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__efc4edd7a9609a122263aded40cd4f4d0868d1199938d86dd5236e67a68d5f92(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, DataTailscaleDevicesFilter]],
) -> None:
    """Type checking stubs"""
    pass
