r'''
# `provider`

Refer to the OpenTofu Registry for docs: [`tailscale`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs).
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


class TailscaleProvider(
    _cdktn_78ede62e.TerraformProvider,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.provider.TailscaleProvider",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs tailscale}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        alias: typing.Optional[builtins.str] = None,
        api_key: typing.Optional[builtins.str] = None,
        audience: typing.Optional[builtins.str] = None,
        base_url: typing.Optional[builtins.str] = None,
        identity_token: typing.Optional[builtins.str] = None,
        identity_token_environment_variable_name: typing.Optional[builtins.str] = None,
        oauth_client_id: typing.Optional[builtins.str] = None,
        oauth_client_secret: typing.Optional[builtins.str] = None,
        scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
        tailnet: typing.Optional[builtins.str] = None,
        user_agent: typing.Optional[builtins.str] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs tailscale} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param alias: Alias name. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#alias TailscaleProvider#alias}
        :param api_key: The API key to use for authenticating requests to the API. Can be set via the TAILSCALE_API_KEY environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the API key. Conflicts with 'oauth_client_id' and 'oauth_client_secret'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#api_key TailscaleProvider#api_key}
        :param audience: The OIDC audience to request when discovering an identity token from the runtime (GitHub Actions, AWS, or GCP) for workload identity federation. Can be set via the TAILSCALE_AUDIENCE environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the audience. Requires 'oauth_client_id'. Conflicts with 'api_key', 'oauth_client_secret', 'identity_token', and 'identity_token_environment_variable_name'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#audience TailscaleProvider#audience}
        :param base_url: The base URL of the Tailscale API. Defaults to https://api.tailscale.com. Can be set via the TAILSCALE_BASE_URL environment variable. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#base_url TailscaleProvider#base_url}
        :param identity_token: The jwt identity token to exchange for a Tailscale API token when using a federated identity. Can be set via the TAILSCALE_IDENTITY_TOKEN environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the identity token. Conflicts with 'api_key', 'oauth_client_secret', and 'identity_token_environment_variable_name'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token TailscaleProvider#identity_token}
        :param identity_token_environment_variable_name: The name of an environment variable to read the identity token from. This is useful when the identity token is provided by an external system (such as Terraform Cloud workload identity) in an environment variable you do not control. If the resolved value of the environment variable starts with 'file:' then it is treated as a path to a file on disk that contains identity token. Conflicts with 'identity_token'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token_environment_variable_name TailscaleProvider#identity_token_environment_variable_name}
        :param oauth_client_id: The OAuth application or federated identity's ID when using OAuth client credentials or workload identity federation. Can be set via the TAILSCALE_OAUTH_CLIENT_ID environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client ID. Either 'oauth_client_secret' or 'identity_token' must be set alongside 'oauth_client_id'. Conflicts with 'api_key'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_id TailscaleProvider#oauth_client_id}
        :param oauth_client_secret: The OAuth application's secret when using OAuth client credentials. Can be set via the TAILSCALE_OAUTH_CLIENT_SECRET environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client secret. Conflicts with 'api_key' and 'identity_token'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_secret TailscaleProvider#oauth_client_secret}
        :param scopes: The OAuth 2.0 scopes to request when generating the access token using the supplied OAuth client credentials. See https://tailscale.com/kb/1623/trust-credentials#scopes for available scopes. Only valid when both 'oauth_client_id' and 'oauth_client_secret', or both are set. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#scopes TailscaleProvider#scopes}
        :param tailnet: The tailnet ID. Tailnets created before Oct 2025 can still use the legacy ID, but the Tailnet ID is the preferred identifier. Can be set via the TAILSCALE_TAILNET environment variable. Default is the tailnet that owns API credentials passed to the provider. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#tailnet TailscaleProvider#tailnet}
        :param user_agent: User-Agent header for API requests. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#user_agent TailscaleProvider#user_agent}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7c00aec8cce6f86aed0b3eb4dad51f61765a8a8826438b8f52cf9aa59897414b)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = TailscaleProviderConfig(
            alias=alias,
            api_key=api_key,
            audience=audience,
            base_url=base_url,
            identity_token=identity_token,
            identity_token_environment_variable_name=identity_token_environment_variable_name,
            oauth_client_id=oauth_client_id,
            oauth_client_secret=oauth_client_secret,
            scopes=scopes,
            tailnet=tailnet,
            user_agent=user_agent,
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
        '''Generates CDKTN code for importing a TailscaleProvider resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the TailscaleProvider to import.
        :param import_from_id: The id of the existing TailscaleProvider that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the TailscaleProvider to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__09d5ed9c151ca1172a198fb61feba3b1d8bb7fa4af2dcc12b961c71b42e44caf)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetAlias")
    def reset_alias(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetAlias", []))

    @jsii.member(jsii_name="resetApiKey")
    def reset_api_key(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetApiKey", []))

    @jsii.member(jsii_name="resetAudience")
    def reset_audience(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetAudience", []))

    @jsii.member(jsii_name="resetBaseUrl")
    def reset_base_url(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetBaseUrl", []))

    @jsii.member(jsii_name="resetIdentityToken")
    def reset_identity_token(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetIdentityToken", []))

    @jsii.member(jsii_name="resetIdentityTokenEnvironmentVariableName")
    def reset_identity_token_environment_variable_name(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetIdentityTokenEnvironmentVariableName", []))

    @jsii.member(jsii_name="resetOauthClientId")
    def reset_oauth_client_id(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetOauthClientId", []))

    @jsii.member(jsii_name="resetOauthClientSecret")
    def reset_oauth_client_secret(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetOauthClientSecret", []))

    @jsii.member(jsii_name="resetScopes")
    def reset_scopes(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetScopes", []))

    @jsii.member(jsii_name="resetTailnet")
    def reset_tailnet(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetTailnet", []))

    @jsii.member(jsii_name="resetUserAgent")
    def reset_user_agent(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUserAgent", []))

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
    @jsii.member(jsii_name="aliasInput")
    def alias_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "aliasInput"))

    @builtins.property
    @jsii.member(jsii_name="apiKeyInput")
    def api_key_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "apiKeyInput"))

    @builtins.property
    @jsii.member(jsii_name="audienceInput")
    def audience_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "audienceInput"))

    @builtins.property
    @jsii.member(jsii_name="baseUrlInput")
    def base_url_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "baseUrlInput"))

    @builtins.property
    @jsii.member(jsii_name="identityTokenEnvironmentVariableNameInput")
    def identity_token_environment_variable_name_input(
        self,
    ) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "identityTokenEnvironmentVariableNameInput"))

    @builtins.property
    @jsii.member(jsii_name="identityTokenInput")
    def identity_token_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "identityTokenInput"))

    @builtins.property
    @jsii.member(jsii_name="oauthClientIdInput")
    def oauth_client_id_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "oauthClientIdInput"))

    @builtins.property
    @jsii.member(jsii_name="oauthClientSecretInput")
    def oauth_client_secret_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "oauthClientSecretInput"))

    @builtins.property
    @jsii.member(jsii_name="scopesInput")
    def scopes_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "scopesInput"))

    @builtins.property
    @jsii.member(jsii_name="tailnetInput")
    def tailnet_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "tailnetInput"))

    @builtins.property
    @jsii.member(jsii_name="userAgentInput")
    def user_agent_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "userAgentInput"))

    @builtins.property
    @jsii.member(jsii_name="alias")
    def alias(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "alias"))

    @alias.setter
    def alias(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__28a1ef6e026c0b0efec0e52551239f89e357f41d5725eb727816da00fe3d4b0b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "alias", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="apiKey")
    def api_key(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "apiKey"))

    @api_key.setter
    def api_key(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__b307f8306a40980cf9a77ec8408594b5ecb65f63e9113f629e8b5be04bd90d43)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "apiKey", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="audience")
    def audience(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "audience"))

    @audience.setter
    def audience(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a2be63c376b07899984eb229bd9377e4b34baabf5b306d363b6ee767bf52e66f)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "audience", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="baseUrl")
    def base_url(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "baseUrl"))

    @base_url.setter
    def base_url(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__eefb49a22c9d645ddc7a14e8f6a69e93ef3ba1aa934139ce950585365c2c025b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "baseUrl", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="identityToken")
    def identity_token(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "identityToken"))

    @identity_token.setter
    def identity_token(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4928bdd2f4bfa5dc992a16b46fb421ae0d167330053adfb7999217c38863aba2)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "identityToken", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="identityTokenEnvironmentVariableName")
    def identity_token_environment_variable_name(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "identityTokenEnvironmentVariableName"))

    @identity_token_environment_variable_name.setter
    def identity_token_environment_variable_name(
        self,
        value: typing.Optional[builtins.str],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__608d7ddc2a66014ef83c0aac93c897de5526e98319f3adeb1917ab74f07f03b7)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "identityTokenEnvironmentVariableName", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="oauthClientId")
    def oauth_client_id(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "oauthClientId"))

    @oauth_client_id.setter
    def oauth_client_id(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7b395deb5a4f94293c75211ebab3672ba7d06c90227d415b60334a7d73870fbe)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "oauthClientId", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="oauthClientSecret")
    def oauth_client_secret(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "oauthClientSecret"))

    @oauth_client_secret.setter
    def oauth_client_secret(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a8964683f0265a3cfe05bd258910ba76dc89f93430d08fee7ce7ad23ee00a299)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "oauthClientSecret", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="scopes")
    def scopes(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "scopes"))

    @scopes.setter
    def scopes(self, value: typing.Optional[typing.List[builtins.str]]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4fca7792b6c7aefa85d03ecdec11733b7db5905b1a4bf5f7a806251ab2c2ff9f)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "scopes", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="tailnet")
    def tailnet(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "tailnet"))

    @tailnet.setter
    def tailnet(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__1f938c9bb67145545278faaec2bfc44a892a56636d58e57ced8ba004008418c7)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "tailnet", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="userAgent")
    def user_agent(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "userAgent"))

    @user_agent.setter
    def user_agent(self, value: typing.Optional[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__6b7bbff08c13d23d4a04abdba30fc9138b4c5ce79bd15033ba1d690cd178ba8b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "userAgent", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.provider.TailscaleProviderConfig",
    jsii_struct_bases=[],
    name_mapping={
        "alias": "alias",
        "api_key": "apiKey",
        "audience": "audience",
        "base_url": "baseUrl",
        "identity_token": "identityToken",
        "identity_token_environment_variable_name": "identityTokenEnvironmentVariableName",
        "oauth_client_id": "oauthClientId",
        "oauth_client_secret": "oauthClientSecret",
        "scopes": "scopes",
        "tailnet": "tailnet",
        "user_agent": "userAgent",
    },
)
class TailscaleProviderConfig:
    def __init__(
        self,
        *,
        alias: typing.Optional[builtins.str] = None,
        api_key: typing.Optional[builtins.str] = None,
        audience: typing.Optional[builtins.str] = None,
        base_url: typing.Optional[builtins.str] = None,
        identity_token: typing.Optional[builtins.str] = None,
        identity_token_environment_variable_name: typing.Optional[builtins.str] = None,
        oauth_client_id: typing.Optional[builtins.str] = None,
        oauth_client_secret: typing.Optional[builtins.str] = None,
        scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
        tailnet: typing.Optional[builtins.str] = None,
        user_agent: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param alias: Alias name. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#alias TailscaleProvider#alias}
        :param api_key: The API key to use for authenticating requests to the API. Can be set via the TAILSCALE_API_KEY environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the API key. Conflicts with 'oauth_client_id' and 'oauth_client_secret'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#api_key TailscaleProvider#api_key}
        :param audience: The OIDC audience to request when discovering an identity token from the runtime (GitHub Actions, AWS, or GCP) for workload identity federation. Can be set via the TAILSCALE_AUDIENCE environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the audience. Requires 'oauth_client_id'. Conflicts with 'api_key', 'oauth_client_secret', 'identity_token', and 'identity_token_environment_variable_name'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#audience TailscaleProvider#audience}
        :param base_url: The base URL of the Tailscale API. Defaults to https://api.tailscale.com. Can be set via the TAILSCALE_BASE_URL environment variable. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#base_url TailscaleProvider#base_url}
        :param identity_token: The jwt identity token to exchange for a Tailscale API token when using a federated identity. Can be set via the TAILSCALE_IDENTITY_TOKEN environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the identity token. Conflicts with 'api_key', 'oauth_client_secret', and 'identity_token_environment_variable_name'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token TailscaleProvider#identity_token}
        :param identity_token_environment_variable_name: The name of an environment variable to read the identity token from. This is useful when the identity token is provided by an external system (such as Terraform Cloud workload identity) in an environment variable you do not control. If the resolved value of the environment variable starts with 'file:' then it is treated as a path to a file on disk that contains identity token. Conflicts with 'identity_token'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token_environment_variable_name TailscaleProvider#identity_token_environment_variable_name}
        :param oauth_client_id: The OAuth application or federated identity's ID when using OAuth client credentials or workload identity federation. Can be set via the TAILSCALE_OAUTH_CLIENT_ID environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client ID. Either 'oauth_client_secret' or 'identity_token' must be set alongside 'oauth_client_id'. Conflicts with 'api_key'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_id TailscaleProvider#oauth_client_id}
        :param oauth_client_secret: The OAuth application's secret when using OAuth client credentials. Can be set via the TAILSCALE_OAUTH_CLIENT_SECRET environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client secret. Conflicts with 'api_key' and 'identity_token'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_secret TailscaleProvider#oauth_client_secret}
        :param scopes: The OAuth 2.0 scopes to request when generating the access token using the supplied OAuth client credentials. See https://tailscale.com/kb/1623/trust-credentials#scopes for available scopes. Only valid when both 'oauth_client_id' and 'oauth_client_secret', or both are set. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#scopes TailscaleProvider#scopes}
        :param tailnet: The tailnet ID. Tailnets created before Oct 2025 can still use the legacy ID, but the Tailnet ID is the preferred identifier. Can be set via the TAILSCALE_TAILNET environment variable. Default is the tailnet that owns API credentials passed to the provider. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#tailnet TailscaleProvider#tailnet}
        :param user_agent: User-Agent header for API requests. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#user_agent TailscaleProvider#user_agent}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__e65ea007127aa66e30126f2ebf7b50131dac0f2dc73bcbf43b438263d3a10518)
            check_type(argname="argument alias", value=alias, expected_type=type_hints["alias"])
            check_type(argname="argument api_key", value=api_key, expected_type=type_hints["api_key"])
            check_type(argname="argument audience", value=audience, expected_type=type_hints["audience"])
            check_type(argname="argument base_url", value=base_url, expected_type=type_hints["base_url"])
            check_type(argname="argument identity_token", value=identity_token, expected_type=type_hints["identity_token"])
            check_type(argname="argument identity_token_environment_variable_name", value=identity_token_environment_variable_name, expected_type=type_hints["identity_token_environment_variable_name"])
            check_type(argname="argument oauth_client_id", value=oauth_client_id, expected_type=type_hints["oauth_client_id"])
            check_type(argname="argument oauth_client_secret", value=oauth_client_secret, expected_type=type_hints["oauth_client_secret"])
            check_type(argname="argument scopes", value=scopes, expected_type=type_hints["scopes"])
            check_type(argname="argument tailnet", value=tailnet, expected_type=type_hints["tailnet"])
            check_type(argname="argument user_agent", value=user_agent, expected_type=type_hints["user_agent"])
        self._values: typing.Dict[builtins.str, typing.Any] = {}
        if alias is not None:
            self._values["alias"] = alias
        if api_key is not None:
            self._values["api_key"] = api_key
        if audience is not None:
            self._values["audience"] = audience
        if base_url is not None:
            self._values["base_url"] = base_url
        if identity_token is not None:
            self._values["identity_token"] = identity_token
        if identity_token_environment_variable_name is not None:
            self._values["identity_token_environment_variable_name"] = identity_token_environment_variable_name
        if oauth_client_id is not None:
            self._values["oauth_client_id"] = oauth_client_id
        if oauth_client_secret is not None:
            self._values["oauth_client_secret"] = oauth_client_secret
        if scopes is not None:
            self._values["scopes"] = scopes
        if tailnet is not None:
            self._values["tailnet"] = tailnet
        if user_agent is not None:
            self._values["user_agent"] = user_agent

    @builtins.property
    def alias(self) -> typing.Optional[builtins.str]:
        '''Alias name.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#alias TailscaleProvider#alias}
        '''
        result = self._values.get("alias")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def api_key(self) -> typing.Optional[builtins.str]:
        '''The API key to use for authenticating requests to the API.

        Can be set via the TAILSCALE_API_KEY environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the API key. Conflicts with 'oauth_client_id' and 'oauth_client_secret'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#api_key TailscaleProvider#api_key}
        '''
        result = self._values.get("api_key")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def audience(self) -> typing.Optional[builtins.str]:
        '''The OIDC audience to request when discovering an identity token from the runtime (GitHub Actions, AWS, or GCP) for workload identity federation.

        Can be set via the TAILSCALE_AUDIENCE environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the audience. Requires 'oauth_client_id'. Conflicts with 'api_key', 'oauth_client_secret', 'identity_token', and 'identity_token_environment_variable_name'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#audience TailscaleProvider#audience}
        '''
        result = self._values.get("audience")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def base_url(self) -> typing.Optional[builtins.str]:
        '''The base URL of the Tailscale API. Defaults to https://api.tailscale.com. Can be set via the TAILSCALE_BASE_URL environment variable.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#base_url TailscaleProvider#base_url}
        '''
        result = self._values.get("base_url")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def identity_token(self) -> typing.Optional[builtins.str]:
        '''The jwt identity token to exchange for a Tailscale API token when using a federated identity.

        Can be set via the TAILSCALE_IDENTITY_TOKEN environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the identity token. Conflicts with 'api_key', 'oauth_client_secret', and 'identity_token_environment_variable_name'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token TailscaleProvider#identity_token}
        '''
        result = self._values.get("identity_token")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def identity_token_environment_variable_name(self) -> typing.Optional[builtins.str]:
        '''The name of an environment variable to read the identity token from.

        This is useful when the identity token is provided by an external system (such as Terraform Cloud workload identity) in an environment variable you do not control. If the resolved value of the environment variable starts with 'file:' then it is treated as a path to a file on disk that contains identity token. Conflicts with 'identity_token'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#identity_token_environment_variable_name TailscaleProvider#identity_token_environment_variable_name}
        '''
        result = self._values.get("identity_token_environment_variable_name")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def oauth_client_id(self) -> typing.Optional[builtins.str]:
        '''The OAuth application or federated identity's ID when using OAuth client credentials or workload identity federation.

        Can be set via the TAILSCALE_OAUTH_CLIENT_ID environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client ID. Either 'oauth_client_secret' or 'identity_token' must be set alongside 'oauth_client_id'. Conflicts with 'api_key'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_id TailscaleProvider#oauth_client_id}
        '''
        result = self._values.get("oauth_client_id")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def oauth_client_secret(self) -> typing.Optional[builtins.str]:
        '''The OAuth application's secret when using OAuth client credentials.

        Can be set via the TAILSCALE_OAUTH_CLIENT_SECRET environment variable. If the value starts with 'file:' then it is treated as a path to a file on disk that contains the client secret. Conflicts with 'api_key' and 'identity_token'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#oauth_client_secret TailscaleProvider#oauth_client_secret}
        '''
        result = self._values.get("oauth_client_secret")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def scopes(self) -> typing.Optional[typing.List[builtins.str]]:
        '''The OAuth 2.0 scopes to request when generating the access token using the supplied OAuth client credentials. See https://tailscale.com/kb/1623/trust-credentials#scopes for available scopes. Only valid when both 'oauth_client_id' and 'oauth_client_secret', or both are set.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#scopes TailscaleProvider#scopes}
        '''
        result = self._values.get("scopes")
        return typing.cast(typing.Optional[typing.List[builtins.str]], result)

    @builtins.property
    def tailnet(self) -> typing.Optional[builtins.str]:
        '''The tailnet ID.

        Tailnets created before Oct 2025 can still use the legacy ID, but the Tailnet ID is the preferred identifier. Can be set via the TAILSCALE_TAILNET environment variable. Default is the tailnet that owns API credentials passed to the provider.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#tailnet TailscaleProvider#tailnet}
        '''
        result = self._values.get("tailnet")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def user_agent(self) -> typing.Optional[builtins.str]:
        '''User-Agent header for API requests.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs#user_agent TailscaleProvider#user_agent}
        '''
        result = self._values.get("user_agent")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "TailscaleProviderConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "TailscaleProvider",
    "TailscaleProviderConfig",
]

publication.publish()

def _typecheckingstub__7c00aec8cce6f86aed0b3eb4dad51f61765a8a8826438b8f52cf9aa59897414b(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    alias: typing.Optional[builtins.str] = None,
    api_key: typing.Optional[builtins.str] = None,
    audience: typing.Optional[builtins.str] = None,
    base_url: typing.Optional[builtins.str] = None,
    identity_token: typing.Optional[builtins.str] = None,
    identity_token_environment_variable_name: typing.Optional[builtins.str] = None,
    oauth_client_id: typing.Optional[builtins.str] = None,
    oauth_client_secret: typing.Optional[builtins.str] = None,
    scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
    tailnet: typing.Optional[builtins.str] = None,
    user_agent: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__09d5ed9c151ca1172a198fb61feba3b1d8bb7fa4af2dcc12b961c71b42e44caf(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__28a1ef6e026c0b0efec0e52551239f89e357f41d5725eb727816da00fe3d4b0b(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__b307f8306a40980cf9a77ec8408594b5ecb65f63e9113f629e8b5be04bd90d43(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__a2be63c376b07899984eb229bd9377e4b34baabf5b306d363b6ee767bf52e66f(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__eefb49a22c9d645ddc7a14e8f6a69e93ef3ba1aa934139ce950585365c2c025b(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4928bdd2f4bfa5dc992a16b46fb421ae0d167330053adfb7999217c38863aba2(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__608d7ddc2a66014ef83c0aac93c897de5526e98319f3adeb1917ab74f07f03b7(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7b395deb5a4f94293c75211ebab3672ba7d06c90227d415b60334a7d73870fbe(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__a8964683f0265a3cfe05bd258910ba76dc89f93430d08fee7ce7ad23ee00a299(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4fca7792b6c7aefa85d03ecdec11733b7db5905b1a4bf5f7a806251ab2c2ff9f(
    value: typing.Optional[typing.List[builtins.str]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__1f938c9bb67145545278faaec2bfc44a892a56636d58e57ced8ba004008418c7(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__6b7bbff08c13d23d4a04abdba30fc9138b4c5ce79bd15033ba1d690cd178ba8b(
    value: typing.Optional[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__e65ea007127aa66e30126f2ebf7b50131dac0f2dc73bcbf43b438263d3a10518(
    *,
    alias: typing.Optional[builtins.str] = None,
    api_key: typing.Optional[builtins.str] = None,
    audience: typing.Optional[builtins.str] = None,
    base_url: typing.Optional[builtins.str] = None,
    identity_token: typing.Optional[builtins.str] = None,
    identity_token_environment_variable_name: typing.Optional[builtins.str] = None,
    oauth_client_id: typing.Optional[builtins.str] = None,
    oauth_client_secret: typing.Optional[builtins.str] = None,
    scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
    tailnet: typing.Optional[builtins.str] = None,
    user_agent: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass
