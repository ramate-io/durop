r'''
# `tailscale_webhook`

Refer to the OpenTofu Registry for docs: [`tailscale_webhook`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook).
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


class Webhook(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.webhook.Webhook",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook tailscale_webhook}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        endpoint_url: builtins.str,
        subscriptions: typing.Sequence[builtins.str],
        provider_type: typing.Optional[builtins.str] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook tailscale_webhook} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param endpoint_url: The endpoint to send webhook events to. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#endpoint_url Webhook#endpoint_url}
        :param subscriptions: The set of events that trigger this webhook. For a full list of event types, see the `webhooks documentation <https://tailscale.com/kb/1213/webhooks#events>`_. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#subscriptions Webhook#subscriptions}
        :param provider_type: The provider type of the endpoint URL. This determines the payload format sent to the destination. Valid values are ``slack``, ``mattermost``, ``googlechat``, and ``discord``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#provider_type Webhook#provider_type}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__8e8b130c4bdd5a91848fa8409b8d0b26aafabb7d52b07e4f43ed96db2d78a47d)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = WebhookConfig(
            endpoint_url=endpoint_url,
            subscriptions=subscriptions,
            provider_type=provider_type,
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
        '''Generates CDKTN code for importing a Webhook resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the Webhook to import.
        :param import_from_id: The id of the existing Webhook that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the Webhook to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7a9fbf433b4b7f7c74f6f3d1c62cd6fcf3d91cd16bef53748939492b7ca60934)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetProviderType")
    def reset_provider_type(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetProviderType", []))

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
    @jsii.member(jsii_name="secret")
    def secret(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "secret"))

    @builtins.property
    @jsii.member(jsii_name="endpointUrlInput")
    def endpoint_url_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "endpointUrlInput"))

    @builtins.property
    @jsii.member(jsii_name="providerTypeInput")
    def provider_type_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "providerTypeInput"))

    @builtins.property
    @jsii.member(jsii_name="subscriptionsInput")
    def subscriptions_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "subscriptionsInput"))

    @builtins.property
    @jsii.member(jsii_name="endpointUrl")
    def endpoint_url(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "endpointUrl"))

    @endpoint_url.setter
    def endpoint_url(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__953a1bae292d487847b7c139f0360ac4d898b6fe4378a65f3fa20f435e9cc598)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "endpointUrl", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="providerType")
    def provider_type(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "providerType"))

    @provider_type.setter
    def provider_type(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__99784bb5ebbc94e6ddb99ff9e1e2f931f7fa21d1285def91e3c5d44ef2730065)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "providerType", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="subscriptions")
    def subscriptions(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "subscriptions"))

    @subscriptions.setter
    def subscriptions(self, value: typing.List[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__69ee4ebcb9f1b9bd18d86d865d161cacfc30709a277d330bde88e560c9f3eb6a)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "subscriptions", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.webhook.WebhookConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "endpoint_url": "endpointUrl",
        "subscriptions": "subscriptions",
        "provider_type": "providerType",
    },
)
class WebhookConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        endpoint_url: builtins.str,
        subscriptions: typing.Sequence[builtins.str],
        provider_type: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param endpoint_url: The endpoint to send webhook events to. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#endpoint_url Webhook#endpoint_url}
        :param subscriptions: The set of events that trigger this webhook. For a full list of event types, see the `webhooks documentation <https://tailscale.com/kb/1213/webhooks#events>`_. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#subscriptions Webhook#subscriptions}
        :param provider_type: The provider type of the endpoint URL. This determines the payload format sent to the destination. Valid values are ``slack``, ``mattermost``, ``googlechat``, and ``discord``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#provider_type Webhook#provider_type}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f585b71b3589a3ef72591f9ab22c638ce07ca1a25c832a0ceb477793d01fc72a)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument endpoint_url", value=endpoint_url, expected_type=type_hints["endpoint_url"])
            check_type(argname="argument subscriptions", value=subscriptions, expected_type=type_hints["subscriptions"])
            check_type(argname="argument provider_type", value=provider_type, expected_type=type_hints["provider_type"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "endpoint_url": endpoint_url,
            "subscriptions": subscriptions,
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
        if provider_type is not None:
            self._values["provider_type"] = provider_type

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
    def endpoint_url(self) -> builtins.str:
        '''The endpoint to send webhook events to.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#endpoint_url Webhook#endpoint_url}
        '''
        result = self._values.get("endpoint_url")
        assert result is not None, "Required property 'endpoint_url' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def subscriptions(self) -> typing.List[builtins.str]:
        '''The set of events that trigger this webhook. For a full list of event types, see the `webhooks documentation <https://tailscale.com/kb/1213/webhooks#events>`_.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#subscriptions Webhook#subscriptions}
        '''
        result = self._values.get("subscriptions")
        assert result is not None, "Required property 'subscriptions' is missing"
        return typing.cast(typing.List[builtins.str], result)

    @builtins.property
    def provider_type(self) -> typing.Optional[builtins.str]:
        '''The provider type of the endpoint URL.

        This determines the payload format sent to the destination. Valid values are ``slack``, ``mattermost``, ``googlechat``, and ``discord``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/webhook#provider_type Webhook#provider_type}
        '''
        result = self._values.get("provider_type")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "WebhookConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "Webhook",
    "WebhookConfig",
]

publication.publish()

def _typecheckingstub__8e8b130c4bdd5a91848fa8409b8d0b26aafabb7d52b07e4f43ed96db2d78a47d(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    endpoint_url: builtins.str,
    subscriptions: typing.Sequence[builtins.str],
    provider_type: typing.Optional[builtins.str] = None,
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

def _typecheckingstub__7a9fbf433b4b7f7c74f6f3d1c62cd6fcf3d91cd16bef53748939492b7ca60934(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__953a1bae292d487847b7c139f0360ac4d898b6fe4378a65f3fa20f435e9cc598(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__99784bb5ebbc94e6ddb99ff9e1e2f931f7fa21d1285def91e3c5d44ef2730065(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__69ee4ebcb9f1b9bd18d86d865d161cacfc30709a277d330bde88e560c9f3eb6a(
    value: typing.List[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f585b71b3589a3ef72591f9ab22c638ce07ca1a25c832a0ceb477793d01fc72a(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    endpoint_url: builtins.str,
    subscriptions: typing.Sequence[builtins.str],
    provider_type: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass
