r'''
# `tailscale_acl`

Refer to the OpenTofu Registry for docs: [`tailscale_acl`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl).
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


class Acl(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.acl.Acl",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl tailscale_acl}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        acl: builtins.str,
        overwrite_existing_content: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        reset_acl_on_destroy: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl tailscale_acl} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param acl: The policy that defines which devices and users are allowed to connect in your network. Can be either a JSON or a HuJSON string. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#acl Acl#acl}
        :param overwrite_existing_content: If true, will skip requirement to import acl before allowing changes. Be careful, can cause the policy file to be overwritten Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#overwrite_existing_content Acl#overwrite_existing_content}
        :param reset_acl_on_destroy: If true, will reset the policy file for the Tailnet to the default when this resource is destroyed. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#reset_acl_on_destroy Acl#reset_acl_on_destroy}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__5428c2f6ae4d8d32b070bff145a05246bb75664330113c2b5f040c1fff36144a)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = AclConfig(
            acl=acl,
            overwrite_existing_content=overwrite_existing_content,
            reset_acl_on_destroy=reset_acl_on_destroy,
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
        '''Generates CDKTN code for importing a Acl resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the Acl to import.
        :param import_from_id: The id of the existing Acl that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the Acl to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__92d576ade869f5c9d75179048bc0dab99dfb4594ab379c81a05c607d202b30c0)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetOverwriteExistingContent")
    def reset_overwrite_existing_content(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetOverwriteExistingContent", []))

    @jsii.member(jsii_name="resetResetAclOnDestroy")
    def reset_reset_acl_on_destroy(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetResetAclOnDestroy", []))

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
    @jsii.member(jsii_name="aclInput")
    def acl_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "aclInput"))

    @builtins.property
    @jsii.member(jsii_name="overwriteExistingContentInput")
    def overwrite_existing_content_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "overwriteExistingContentInput"))

    @builtins.property
    @jsii.member(jsii_name="resetAclOnDestroyInput")
    def reset_acl_on_destroy_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "resetAclOnDestroyInput"))

    @builtins.property
    @jsii.member(jsii_name="acl")
    def acl(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "acl"))

    @acl.setter
    def acl(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__b5341194bfd8167868cd02f376a2c0cbd937d215b99afbf5d2f50a9caace5e19)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "acl", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="overwriteExistingContent")
    def overwrite_existing_content(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "overwriteExistingContent"))

    @overwrite_existing_content.setter
    def overwrite_existing_content(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__d6c30d3580aece4844c22c87595373642c17796cf11bc47d29d6a1b6269d21c2)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "overwriteExistingContent", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="resetAclOnDestroy")
    def reset_acl_on_destroy(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "resetAclOnDestroy"))

    @reset_acl_on_destroy.setter
    def reset_acl_on_destroy(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__421ac17eecdd816115ecca3761804098c63c7144f1e1e5d6fce020a2f3d04cc5)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "resetAclOnDestroy", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.acl.AclConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "acl": "acl",
        "overwrite_existing_content": "overwriteExistingContent",
        "reset_acl_on_destroy": "resetAclOnDestroy",
    },
)
class AclConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        acl: builtins.str,
        overwrite_existing_content: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        reset_acl_on_destroy: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param acl: The policy that defines which devices and users are allowed to connect in your network. Can be either a JSON or a HuJSON string. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#acl Acl#acl}
        :param overwrite_existing_content: If true, will skip requirement to import acl before allowing changes. Be careful, can cause the policy file to be overwritten Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#overwrite_existing_content Acl#overwrite_existing_content}
        :param reset_acl_on_destroy: If true, will reset the policy file for the Tailnet to the default when this resource is destroyed. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#reset_acl_on_destroy Acl#reset_acl_on_destroy}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__2d42ec63f1a1fdf8c83e1cdb4ab099592392341743a3257e8566bf76661b620a)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument acl", value=acl, expected_type=type_hints["acl"])
            check_type(argname="argument overwrite_existing_content", value=overwrite_existing_content, expected_type=type_hints["overwrite_existing_content"])
            check_type(argname="argument reset_acl_on_destroy", value=reset_acl_on_destroy, expected_type=type_hints["reset_acl_on_destroy"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "acl": acl,
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
        if overwrite_existing_content is not None:
            self._values["overwrite_existing_content"] = overwrite_existing_content
        if reset_acl_on_destroy is not None:
            self._values["reset_acl_on_destroy"] = reset_acl_on_destroy

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
    def acl(self) -> builtins.str:
        '''The policy that defines which devices and users are allowed to connect in your network.

        Can be either a JSON or a HuJSON string.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#acl Acl#acl}
        '''
        result = self._values.get("acl")
        assert result is not None, "Required property 'acl' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def overwrite_existing_content(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''If true, will skip requirement to import acl before allowing changes.

        Be careful, can cause the policy file to be overwritten

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#overwrite_existing_content Acl#overwrite_existing_content}
        '''
        result = self._values.get("overwrite_existing_content")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def reset_acl_on_destroy(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''If true, will reset the policy file for the Tailnet to the default when this resource is destroyed.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/acl#reset_acl_on_destroy Acl#reset_acl_on_destroy}
        '''
        result = self._values.get("reset_acl_on_destroy")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "AclConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "Acl",
    "AclConfig",
]

publication.publish()

def _typecheckingstub__5428c2f6ae4d8d32b070bff145a05246bb75664330113c2b5f040c1fff36144a(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    acl: builtins.str,
    overwrite_existing_content: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    reset_acl_on_destroy: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
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

def _typecheckingstub__92d576ade869f5c9d75179048bc0dab99dfb4594ab379c81a05c607d202b30c0(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__b5341194bfd8167868cd02f376a2c0cbd937d215b99afbf5d2f50a9caace5e19(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__d6c30d3580aece4844c22c87595373642c17796cf11bc47d29d6a1b6269d21c2(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__421ac17eecdd816115ecca3761804098c63c7144f1e1e5d6fce020a2f3d04cc5(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__2d42ec63f1a1fdf8c83e1cdb4ab099592392341743a3257e8566bf76661b620a(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    acl: builtins.str,
    overwrite_existing_content: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    reset_acl_on_destroy: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
) -> None:
    """Type checking stubs"""
    pass
