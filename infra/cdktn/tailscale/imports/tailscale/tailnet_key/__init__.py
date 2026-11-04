r'''
# `tailscale_tailnet_key`

Refer to the OpenTofu Registry for docs: [`tailscale_tailnet_key`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key).
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


class TailnetKey(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.tailnetKey.TailnetKey",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key tailscale_tailnet_key}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        description: typing.Optional[builtins.str] = None,
        ephemeral: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        expiry: typing.Optional[jsii.Number] = None,
        preauthorized: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        recreate_if_invalid: typing.Optional[builtins.str] = None,
        reusable: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        tags: typing.Optional[typing.Sequence[builtins.str]] = None,
        user_id: typing.Optional[builtins.str] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key tailscale_tailnet_key} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param description: A description of the key consisting of alphanumeric characters. Defaults to ``""``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#description TailnetKey#description}
        :param ephemeral: Indicates if the key is ephemeral. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#ephemeral TailnetKey#ephemeral}
        :param expiry: The expiry of the key in seconds. Defaults to ``7776000`` (90 days). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#expiry TailnetKey#expiry}
        :param preauthorized: Determines whether or not the machines authenticated by the key will be authorized for the tailnet by default. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#preauthorized TailnetKey#preauthorized}
        :param recreate_if_invalid: Determines whether the key should be created again if it becomes invalid. By default, reusable keys will be recreated, but single-use keys will not. Possible values: 'always', 'never'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#recreate_if_invalid TailnetKey#recreate_if_invalid}
        :param reusable: Indicates if the key is reusable or single-use. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#reusable TailnetKey#reusable}
        :param tags: List of tags to apply to the machines authenticated by the key. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#tags TailnetKey#tags}
        :param user_id: ID of the user who created this key, empty for keys created by OAuth clients. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#user_id TailnetKey#user_id}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__84860a2dc5c2fcf0c846d61e330eb60d70f43b7c4e4e403117fbeb0c118dcf8a)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = TailnetKeyConfig(
            description=description,
            ephemeral=ephemeral,
            expiry=expiry,
            preauthorized=preauthorized,
            recreate_if_invalid=recreate_if_invalid,
            reusable=reusable,
            tags=tags,
            user_id=user_id,
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
        '''Generates CDKTN code for importing a TailnetKey resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the TailnetKey to import.
        :param import_from_id: The id of the existing TailnetKey that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the TailnetKey to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__24907678475f1d28a9363590be08944afdac0e6397d06fd64d615ed6fc7b0517)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetDescription")
    def reset_description(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetDescription", []))

    @jsii.member(jsii_name="resetEphemeral")
    def reset_ephemeral(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetEphemeral", []))

    @jsii.member(jsii_name="resetExpiry")
    def reset_expiry(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetExpiry", []))

    @jsii.member(jsii_name="resetPreauthorized")
    def reset_preauthorized(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetPreauthorized", []))

    @jsii.member(jsii_name="resetRecreateIfInvalid")
    def reset_recreate_if_invalid(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetRecreateIfInvalid", []))

    @jsii.member(jsii_name="resetReusable")
    def reset_reusable(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetReusable", []))

    @jsii.member(jsii_name="resetTags")
    def reset_tags(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetTags", []))

    @jsii.member(jsii_name="resetUserId")
    def reset_user_id(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUserId", []))

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
    @jsii.member(jsii_name="createdAt")
    def created_at(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "createdAt"))

    @builtins.property
    @jsii.member(jsii_name="expiresAt")
    def expires_at(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "expiresAt"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="invalid")
    def invalid(self) -> "_cdktn_78ede62e.IResolvable":
        return typing.cast("_cdktn_78ede62e.IResolvable", jsii.get(self, "invalid"))

    @builtins.property
    @jsii.member(jsii_name="key")
    def key(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "key"))

    @builtins.property
    @jsii.member(jsii_name="descriptionInput")
    def description_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "descriptionInput"))

    @builtins.property
    @jsii.member(jsii_name="ephemeralInput")
    def ephemeral_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "ephemeralInput"))

    @builtins.property
    @jsii.member(jsii_name="expiryInput")
    def expiry_input(self) -> typing.Optional[jsii.Number]:
        return typing.cast(typing.Optional[jsii.Number], jsii.get(self, "expiryInput"))

    @builtins.property
    @jsii.member(jsii_name="preauthorizedInput")
    def preauthorized_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "preauthorizedInput"))

    @builtins.property
    @jsii.member(jsii_name="recreateIfInvalidInput")
    def recreate_if_invalid_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "recreateIfInvalidInput"))

    @builtins.property
    @jsii.member(jsii_name="reusableInput")
    def reusable_input(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], jsii.get(self, "reusableInput"))

    @builtins.property
    @jsii.member(jsii_name="tagsInput")
    def tags_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "tagsInput"))

    @builtins.property
    @jsii.member(jsii_name="userIdInput")
    def user_id_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "userIdInput"))

    @builtins.property
    @jsii.member(jsii_name="description")
    def description(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "description"))

    @description.setter
    def description(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__9ecac84d7910daf20e105c92b293f5cb4c1849c27f81a8cbdf192862a88f1599)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "description", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="ephemeral")
    def ephemeral(self) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "ephemeral"))

    @ephemeral.setter
    def ephemeral(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__5d339c550f6d0c772041dfed0ce0bd0bc147fa907a90a8e221b88c9ce9ff4352)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "ephemeral", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="expiry")
    def expiry(self) -> jsii.Number:
        return typing.cast(jsii.Number, jsii.get(self, "expiry"))

    @expiry.setter
    def expiry(self, value: jsii.Number) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c2f465f71827b1c73903fd2f1e3757a882ebba4ea84d8d6d073efb86a28f6d43)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "expiry", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="preauthorized")
    def preauthorized(
        self,
    ) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "preauthorized"))

    @preauthorized.setter
    def preauthorized(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4598fd5ea1cafa794cb7152a0c1007bb0e6387d5ef0ce580b252b028a5e6bef5)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "preauthorized", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="recreateIfInvalid")
    def recreate_if_invalid(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "recreateIfInvalid"))

    @recreate_if_invalid.setter
    def recreate_if_invalid(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__92a139dd9ace14af82c72463a378533df9d7e1c67400a39944b098706ca270ce)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "recreateIfInvalid", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="reusable")
    def reusable(self) -> typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]:
        return typing.cast(typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"], jsii.get(self, "reusable"))

    @reusable.setter
    def reusable(
        self,
        value: typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__617a230d59ee95b1da7adfbfd5c0dfbf7584077987935ba8dff58dbf29759e60)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "reusable", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="tags")
    def tags(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "tags"))

    @tags.setter
    def tags(self, value: typing.List[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__1a986a76759ac4f9fc262b3bda79111d3fa2096b7529534bac92fff6b042808e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "tags", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="userId")
    def user_id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "userId"))

    @user_id.setter
    def user_id(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__51958f3584d1c5306d92ea8d1f27d3f8f2d32b6b682fedfcdbc99b8bc0783a5b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "userId", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.tailnetKey.TailnetKeyConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "description": "description",
        "ephemeral": "ephemeral",
        "expiry": "expiry",
        "preauthorized": "preauthorized",
        "recreate_if_invalid": "recreateIfInvalid",
        "reusable": "reusable",
        "tags": "tags",
        "user_id": "userId",
    },
)
class TailnetKeyConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        description: typing.Optional[builtins.str] = None,
        ephemeral: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        expiry: typing.Optional[jsii.Number] = None,
        preauthorized: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        recreate_if_invalid: typing.Optional[builtins.str] = None,
        reusable: typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]] = None,
        tags: typing.Optional[typing.Sequence[builtins.str]] = None,
        user_id: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param description: A description of the key consisting of alphanumeric characters. Defaults to ``""``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#description TailnetKey#description}
        :param ephemeral: Indicates if the key is ephemeral. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#ephemeral TailnetKey#ephemeral}
        :param expiry: The expiry of the key in seconds. Defaults to ``7776000`` (90 days). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#expiry TailnetKey#expiry}
        :param preauthorized: Determines whether or not the machines authenticated by the key will be authorized for the tailnet by default. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#preauthorized TailnetKey#preauthorized}
        :param recreate_if_invalid: Determines whether the key should be created again if it becomes invalid. By default, reusable keys will be recreated, but single-use keys will not. Possible values: 'always', 'never'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#recreate_if_invalid TailnetKey#recreate_if_invalid}
        :param reusable: Indicates if the key is reusable or single-use. Defaults to ``false``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#reusable TailnetKey#reusable}
        :param tags: List of tags to apply to the machines authenticated by the key. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#tags TailnetKey#tags}
        :param user_id: ID of the user who created this key, empty for keys created by OAuth clients. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#user_id TailnetKey#user_id}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__aca2f9e99bee9089e2e13b511b1af5d26042fabe2e3e83911f81fcb894663957)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument description", value=description, expected_type=type_hints["description"])
            check_type(argname="argument ephemeral", value=ephemeral, expected_type=type_hints["ephemeral"])
            check_type(argname="argument expiry", value=expiry, expected_type=type_hints["expiry"])
            check_type(argname="argument preauthorized", value=preauthorized, expected_type=type_hints["preauthorized"])
            check_type(argname="argument recreate_if_invalid", value=recreate_if_invalid, expected_type=type_hints["recreate_if_invalid"])
            check_type(argname="argument reusable", value=reusable, expected_type=type_hints["reusable"])
            check_type(argname="argument tags", value=tags, expected_type=type_hints["tags"])
            check_type(argname="argument user_id", value=user_id, expected_type=type_hints["user_id"])
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
        if description is not None:
            self._values["description"] = description
        if ephemeral is not None:
            self._values["ephemeral"] = ephemeral
        if expiry is not None:
            self._values["expiry"] = expiry
        if preauthorized is not None:
            self._values["preauthorized"] = preauthorized
        if recreate_if_invalid is not None:
            self._values["recreate_if_invalid"] = recreate_if_invalid
        if reusable is not None:
            self._values["reusable"] = reusable
        if tags is not None:
            self._values["tags"] = tags
        if user_id is not None:
            self._values["user_id"] = user_id

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
    def description(self) -> typing.Optional[builtins.str]:
        '''A description of the key consisting of alphanumeric characters. Defaults to ``""``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#description TailnetKey#description}
        '''
        result = self._values.get("description")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def ephemeral(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Indicates if the key is ephemeral. Defaults to ``false``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#ephemeral TailnetKey#ephemeral}
        '''
        result = self._values.get("ephemeral")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def expiry(self) -> typing.Optional[jsii.Number]:
        '''The expiry of the key in seconds. Defaults to ``7776000`` (90 days).

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#expiry TailnetKey#expiry}
        '''
        result = self._values.get("expiry")
        return typing.cast(typing.Optional[jsii.Number], result)

    @builtins.property
    def preauthorized(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Determines whether or not the machines authenticated by the key will be authorized for the tailnet by default.

        Defaults to ``false``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#preauthorized TailnetKey#preauthorized}
        '''
        result = self._values.get("preauthorized")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def recreate_if_invalid(self) -> typing.Optional[builtins.str]:
        '''Determines whether the key should be created again if it becomes invalid.

        By default, reusable keys will be recreated, but single-use keys will not. Possible values: 'always', 'never'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#recreate_if_invalid TailnetKey#recreate_if_invalid}
        '''
        result = self._values.get("recreate_if_invalid")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def reusable(
        self,
    ) -> typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]]:
        '''Indicates if the key is reusable or single-use. Defaults to ``false``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#reusable TailnetKey#reusable}
        '''
        result = self._values.get("reusable")
        return typing.cast(typing.Optional[typing.Union[builtins.bool, "_cdktn_78ede62e.IResolvable"]], result)

    @builtins.property
    def tags(self) -> typing.Optional[typing.List[builtins.str]]:
        '''List of tags to apply to the machines authenticated by the key.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#tags TailnetKey#tags}
        '''
        result = self._values.get("tags")
        return typing.cast(typing.Optional[typing.List[builtins.str]], result)

    @builtins.property
    def user_id(self) -> typing.Optional[builtins.str]:
        '''ID of the user who created this key, empty for keys created by OAuth clients.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/tailnet_key#user_id TailnetKey#user_id}
        '''
        result = self._values.get("user_id")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "TailnetKeyConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "TailnetKey",
    "TailnetKeyConfig",
]

publication.publish()

def _typecheckingstub__84860a2dc5c2fcf0c846d61e330eb60d70f43b7c4e4e403117fbeb0c118dcf8a(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    description: typing.Optional[builtins.str] = None,
    ephemeral: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    expiry: typing.Optional[jsii.Number] = None,
    preauthorized: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    recreate_if_invalid: typing.Optional[builtins.str] = None,
    reusable: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    tags: typing.Optional[typing.Sequence[builtins.str]] = None,
    user_id: typing.Optional[builtins.str] = None,
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

def _typecheckingstub__24907678475f1d28a9363590be08944afdac0e6397d06fd64d615ed6fc7b0517(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__9ecac84d7910daf20e105c92b293f5cb4c1849c27f81a8cbdf192862a88f1599(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__5d339c550f6d0c772041dfed0ce0bd0bc147fa907a90a8e221b88c9ce9ff4352(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c2f465f71827b1c73903fd2f1e3757a882ebba4ea84d8d6d073efb86a28f6d43(
    value: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4598fd5ea1cafa794cb7152a0c1007bb0e6387d5ef0ce580b252b028a5e6bef5(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__92a139dd9ace14af82c72463a378533df9d7e1c67400a39944b098706ca270ce(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__617a230d59ee95b1da7adfbfd5c0dfbf7584077987935ba8dff58dbf29759e60(
    value: typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__1a986a76759ac4f9fc262b3bda79111d3fa2096b7529534bac92fff6b042808e(
    value: typing.List[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__51958f3584d1c5306d92ea8d1f27d3f8f2d32b6b682fedfcdbc99b8bc0783a5b(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__aca2f9e99bee9089e2e13b511b1af5d26042fabe2e3e83911f81fcb894663957(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    description: typing.Optional[builtins.str] = None,
    ephemeral: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    expiry: typing.Optional[jsii.Number] = None,
    preauthorized: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    recreate_if_invalid: typing.Optional[builtins.str] = None,
    reusable: typing.Optional[typing.Union[builtins.bool, _cdktn_78ede62e.IResolvable]] = None,
    tags: typing.Optional[typing.Sequence[builtins.str]] = None,
    user_id: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass
