r'''
# `tailscale_contacts`

Refer to the OpenTofu Registry for docs: [`tailscale_contacts`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts).
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


class Contacts(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.Contacts",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts tailscale_contacts}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        account: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsAccount", typing.Dict[builtins.str, typing.Any]]]]] = None,
        security: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSecurity", typing.Dict[builtins.str, typing.Any]]]]] = None,
        support: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSupport", typing.Dict[builtins.str, typing.Any]]]]] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts tailscale_contacts} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param account: account block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#account Contacts#account}
        :param security: security block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#security Contacts#security}
        :param support: support block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#support Contacts#support}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a965de12f2851f98644843aff1919ee189eb43322af24d9d1a2b07100966bac0)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = ContactsConfig(
            account=account,
            security=security,
            support=support,
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
        '''Generates CDKTN code for importing a Contacts resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the Contacts to import.
        :param import_from_id: The id of the existing Contacts that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the Contacts to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__e21f384505fe28bfd9f2fd63dda837353e54d32622bb3e4865b3aaed11f2001f)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="putAccount")
    def put_account(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsAccount", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__6481835102223861d8ec1c296ad3d7d3260e20d018cdea4fdba7be7a360387c8)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putAccount", [value]))

    @jsii.member(jsii_name="putSecurity")
    def put_security(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSecurity", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__31baf47754189ebc171e0de4c50f5ee96dbad4f5e1beb01ce6894ceadcf006bc)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putSecurity", [value]))

    @jsii.member(jsii_name="putSupport")
    def put_support(
        self,
        value: typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSupport", typing.Dict[builtins.str, typing.Any]]]],
    ) -> None:
        '''
        :param value: -
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__b75e03c9972774757d76983f45a874e48a4e8bb7578ab2b1aaab901ddb7279ff)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        return typing.cast(None, jsii.invoke(self, "putSupport", [value]))

    @jsii.member(jsii_name="resetAccount")
    def reset_account(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetAccount", []))

    @jsii.member(jsii_name="resetSecurity")
    def reset_security(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetSecurity", []))

    @jsii.member(jsii_name="resetSupport")
    def reset_support(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetSupport", []))

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
    @jsii.member(jsii_name="account")
    def account(self) -> "ContactsAccountList":
        return typing.cast("ContactsAccountList", jsii.get(self, "account"))

    @builtins.property
    @jsii.member(jsii_name="id")
    def id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "id"))

    @builtins.property
    @jsii.member(jsii_name="security")
    def security(self) -> "ContactsSecurityList":
        return typing.cast("ContactsSecurityList", jsii.get(self, "security"))

    @builtins.property
    @jsii.member(jsii_name="support")
    def support(self) -> "ContactsSupportList":
        return typing.cast("ContactsSupportList", jsii.get(self, "support"))

    @builtins.property
    @jsii.member(jsii_name="accountInput")
    def account_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]], jsii.get(self, "accountInput"))

    @builtins.property
    @jsii.member(jsii_name="securityInput")
    def security_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]], jsii.get(self, "securityInput"))

    @builtins.property
    @jsii.member(jsii_name="supportInput")
    def support_input(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]], jsii.get(self, "supportInput"))


@jsii.data_type(
    jsii_type="tailscale.contacts.ContactsAccount",
    jsii_struct_bases=[],
    name_mapping={"email": "email"},
)
class ContactsAccount:
    def __init__(self, *, email: builtins.str) -> None:
        '''
        :param email: Email address to send communications to. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__78d627c2242d26104d5db148bbdb24161345df1353797425620b064416a5e876)
            check_type(argname="argument email", value=email, expected_type=type_hints["email"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "email": email,
        }

    @builtins.property
    def email(self) -> builtins.str:
        '''Email address to send communications to.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        result = self._values.get("email")
        assert result is not None, "Required property 'email' is missing"
        return typing.cast(builtins.str, result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "ContactsAccount(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class ContactsAccountList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsAccountList",
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
            type_hints = cached_type_hints(_typecheckingstub__42d1ee121611569b55c425953d1a05dc6ede2e1eeff1d530880b5e4ea1e3cdcd)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "ContactsAccountOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__2d358f81bcd774b0a4fe838049f37d73b49c32a4f4d834fda89b8a0d34834979)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("ContactsAccountOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__8e9dca99ddc6434689149dbe8c0c1baf82e40d59f7d33244e4b7ad958bb642a9)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class ContactsAccountOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsAccountOutputReference",
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
            type_hints = cached_type_hints(_typecheckingstub__aa5bdc98426f34e0447b00285d9a05480a72bd46870385823c79cca041ecfc27)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @builtins.property
    @jsii.member(jsii_name="emailInput")
    def email_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "emailInput"))

    @builtins.property
    @jsii.member(jsii_name="email")
    def email(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "email"))

    @email.setter
    def email(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__a1cba7ef91730d1886d748bc160fd40412bcabc784cbfdad42ca71cd0109a59c)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "email", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsAccount"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsAccount"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsAccount"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__e5ff3c50f6edfc052f4d817f9928f072f4c0143b6f9b772a8c0f3b8bacd963d2)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.contacts.ContactsConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "account": "account",
        "security": "security",
        "support": "support",
    },
)
class ContactsConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        account: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsAccount", typing.Dict[builtins.str, typing.Any]]]]] = None,
        security: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSecurity", typing.Dict[builtins.str, typing.Any]]]]] = None,
        support: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.Sequence[typing.Union["ContactsSupport", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param account: account block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#account Contacts#account}
        :param security: security block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#security Contacts#security}
        :param support: support block. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#support Contacts#support}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f4f2bc17421d2c480ee62580d6999618f8430f78a8b6dd5b4280dae965175f00)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument account", value=account, expected_type=type_hints["account"])
            check_type(argname="argument security", value=security, expected_type=type_hints["security"])
            check_type(argname="argument support", value=support, expected_type=type_hints["support"])
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
        if account is not None:
            self._values["account"] = account
        if security is not None:
            self._values["security"] = security
        if support is not None:
            self._values["support"] = support

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
    def account(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]]:
        '''account block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#account Contacts#account}
        '''
        result = self._values.get("account")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsAccount"]]], result)

    @builtins.property
    def security(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]]:
        '''security block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#security Contacts#security}
        '''
        result = self._values.get("security")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]], result)

    @builtins.property
    def support(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]]:
        '''support block.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#support Contacts#support}
        '''
        result = self._values.get("support")
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "ContactsConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


@jsii.data_type(
    jsii_type="tailscale.contacts.ContactsSecurity",
    jsii_struct_bases=[],
    name_mapping={"email": "email"},
)
class ContactsSecurity:
    def __init__(self, *, email: builtins.str) -> None:
        '''
        :param email: Email address to send communications to. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7163af207db0fec9c9bcfe5952d7400cc951bb12d5b067cb459e4548a3552102)
            check_type(argname="argument email", value=email, expected_type=type_hints["email"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "email": email,
        }

    @builtins.property
    def email(self) -> builtins.str:
        '''Email address to send communications to.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        result = self._values.get("email")
        assert result is not None, "Required property 'email' is missing"
        return typing.cast(builtins.str, result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "ContactsSecurity(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class ContactsSecurityList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsSecurityList",
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
            type_hints = cached_type_hints(_typecheckingstub__dd0eae9bdfe95e6db4424a670f8d0aeac717e384d2b8dcd363e37850db2b0465)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "ContactsSecurityOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__f4e3e959c37729b222cecb56c64fc53555a5edcde17eadba6a2fb9df88d39c4e)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("ContactsSecurityOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSecurity"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__18a06d57967b983a605a41fb961798f4d675001dbf9a9b660f01462e43882233)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class ContactsSecurityOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsSecurityOutputReference",
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
            type_hints = cached_type_hints(_typecheckingstub__358e89fc55365518e5bbf8fdac001739e8dd7d0f933b8ccad67aad1a97dbee84)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @builtins.property
    @jsii.member(jsii_name="emailInput")
    def email_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "emailInput"))

    @builtins.property
    @jsii.member(jsii_name="email")
    def email(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "email"))

    @email.setter
    def email(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__384d3bed8c7618e1a91c106add4d61f105869c7c793f5649dcafaf351c9fdf12)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "email", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSecurity"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSecurity"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSecurity"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__0a6341aa285870bd11f9e210d117d718f2baf922817e21bcbfd634b6345500f1)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.contacts.ContactsSupport",
    jsii_struct_bases=[],
    name_mapping={"email": "email"},
)
class ContactsSupport:
    def __init__(self, *, email: builtins.str) -> None:
        '''
        :param email: Email address to send communications to. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__31295351fe103c50a5d6bbecdc9c70874d774839a1d250ea611826f64e4cf25a)
            check_type(argname="argument email", value=email, expected_type=type_hints["email"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "email": email,
        }

    @builtins.property
    def email(self) -> builtins.str:
        '''Email address to send communications to.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/contacts#email Contacts#email}
        '''
        result = self._values.get("email")
        assert result is not None, "Required property 'email' is missing"
        return typing.cast(builtins.str, result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "ContactsSupport(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


class ContactsSupportList(
    _cdktn_78ede62e.ComplexList,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsSupportList",
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
            type_hints = cached_type_hints(_typecheckingstub__7dff078f219004616ae728b6e26ba3a85fa4163a4d84919c76f666320f47f9df)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument wraps_set", value=wraps_set, expected_type=type_hints["wraps_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, wraps_set])

    @jsii.member(jsii_name="get")
    def get(self, index: jsii.Number) -> "ContactsSupportOutputReference":
        '''
        :param index: the index of the item to return.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__407b98378c07ad2f4115e81c631357b97e20de21dfb12408a4c6871744335596)
            check_type(argname="argument index", value=index, expected_type=type_hints["index"])
        return typing.cast("ContactsSupportOutputReference", jsii.invoke(self, "get", [index]))

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", typing.List["ContactsSupport"]]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__780e3eba3f560e299ca546e339ff7670bff64b49e6626ea1f964847811ee12b8)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


class ContactsSupportOutputReference(
    _cdktn_78ede62e.ComplexObject,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.contacts.ContactsSupportOutputReference",
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
            type_hints = cached_type_hints(_typecheckingstub__17e8e5a37745fb6d736bc6df702994a4771b9c060a4f462ad8dac24ff248dc5e)
            check_type(argname="argument terraform_resource", value=terraform_resource, expected_type=type_hints["terraform_resource"])
            check_type(argname="argument terraform_attribute", value=terraform_attribute, expected_type=type_hints["terraform_attribute"])
            check_type(argname="argument complex_object_index", value=complex_object_index, expected_type=type_hints["complex_object_index"])
            check_type(argname="argument complex_object_is_from_set", value=complex_object_is_from_set, expected_type=type_hints["complex_object_is_from_set"])
        jsii.create(self.__class__, self, [terraform_resource, terraform_attribute, complex_object_index, complex_object_is_from_set])

    @builtins.property
    @jsii.member(jsii_name="emailInput")
    def email_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "emailInput"))

    @builtins.property
    @jsii.member(jsii_name="email")
    def email(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "email"))

    @email.setter
    def email(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__0305dd594cc996595d9be319c11b741c8e3b7a55ff1c6dc5204f0115e23468c5)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "email", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="internalValue")
    def internal_value(
        self,
    ) -> typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSupport"]]:
        return typing.cast(typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSupport"]], jsii.get(self, "internalValue"))

    @internal_value.setter
    def internal_value(
        self,
        value: typing.Optional[typing.Union["_cdktn_78ede62e.IResolvable", "ContactsSupport"]],
    ) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c7063fcc1cbcd70dd2ebd9c765b2a52bf35b46289a19ae256c8b00e9f726e3ca)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "internalValue", value) # pyright: ignore[reportArgumentType]


__all__ = [
    "Contacts",
    "ContactsAccount",
    "ContactsAccountList",
    "ContactsAccountOutputReference",
    "ContactsConfig",
    "ContactsSecurity",
    "ContactsSecurityList",
    "ContactsSecurityOutputReference",
    "ContactsSupport",
    "ContactsSupportList",
    "ContactsSupportOutputReference",
]

publication.publish()

def _typecheckingstub__a965de12f2851f98644843aff1919ee189eb43322af24d9d1a2b07100966bac0(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    account: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsAccount, typing.Dict[builtins.str, typing.Any]]]]] = None,
    security: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSecurity, typing.Dict[builtins.str, typing.Any]]]]] = None,
    support: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSupport, typing.Dict[builtins.str, typing.Any]]]]] = None,
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

def _typecheckingstub__e21f384505fe28bfd9f2fd63dda837353e54d32622bb3e4865b3aaed11f2001f(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__6481835102223861d8ec1c296ad3d7d3260e20d018cdea4fdba7be7a360387c8(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsAccount, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__31baf47754189ebc171e0de4c50f5ee96dbad4f5e1beb01ce6894ceadcf006bc(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSecurity, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__b75e03c9972774757d76983f45a874e48a4e8bb7578ab2b1aaab901ddb7279ff(
    value: typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSupport, typing.Dict[builtins.str, typing.Any]]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__78d627c2242d26104d5db148bbdb24161345df1353797425620b064416a5e876(
    *,
    email: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__42d1ee121611569b55c425953d1a05dc6ede2e1eeff1d530880b5e4ea1e3cdcd(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__2d358f81bcd774b0a4fe838049f37d73b49c32a4f4d834fda89b8a0d34834979(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__8e9dca99ddc6434689149dbe8c0c1baf82e40d59f7d33244e4b7ad958bb642a9(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[ContactsAccount]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__aa5bdc98426f34e0447b00285d9a05480a72bd46870385823c79cca041ecfc27(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__a1cba7ef91730d1886d748bc160fd40412bcabc784cbfdad42ca71cd0109a59c(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__e5ff3c50f6edfc052f4d817f9928f072f4c0143b6f9b772a8c0f3b8bacd963d2(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, ContactsAccount]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f4f2bc17421d2c480ee62580d6999618f8430f78a8b6dd5b4280dae965175f00(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    account: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsAccount, typing.Dict[builtins.str, typing.Any]]]]] = None,
    security: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSecurity, typing.Dict[builtins.str, typing.Any]]]]] = None,
    support: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.Sequence[typing.Union[ContactsSupport, typing.Dict[builtins.str, typing.Any]]]]] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7163af207db0fec9c9bcfe5952d7400cc951bb12d5b067cb459e4548a3552102(
    *,
    email: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__dd0eae9bdfe95e6db4424a670f8d0aeac717e384d2b8dcd363e37850db2b0465(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__f4e3e959c37729b222cecb56c64fc53555a5edcde17eadba6a2fb9df88d39c4e(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__18a06d57967b983a605a41fb961798f4d675001dbf9a9b660f01462e43882233(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[ContactsSecurity]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__358e89fc55365518e5bbf8fdac001739e8dd7d0f933b8ccad67aad1a97dbee84(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__384d3bed8c7618e1a91c106add4d61f105869c7c793f5649dcafaf351c9fdf12(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__0a6341aa285870bd11f9e210d117d718f2baf922817e21bcbfd634b6345500f1(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, ContactsSecurity]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__31295351fe103c50a5d6bbecdc9c70874d774839a1d250ea611826f64e4cf25a(
    *,
    email: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7dff078f219004616ae728b6e26ba3a85fa4163a4d84919c76f666320f47f9df(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    wraps_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__407b98378c07ad2f4115e81c631357b97e20de21dfb12408a4c6871744335596(
    index: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__780e3eba3f560e299ca546e339ff7670bff64b49e6626ea1f964847811ee12b8(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, typing.List[ContactsSupport]]],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__17e8e5a37745fb6d736bc6df702994a4771b9c060a4f462ad8dac24ff248dc5e(
    terraform_resource: _cdktn_78ede62e.IInterpolatingParent,
    terraform_attribute: builtins.str,
    complex_object_index: jsii.Number,
    complex_object_is_from_set: builtins.bool,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__0305dd594cc996595d9be319c11b741c8e3b7a55ff1c6dc5204f0115e23468c5(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c7063fcc1cbcd70dd2ebd9c765b2a52bf35b46289a19ae256c8b00e9f726e3ca(
    value: typing.Optional[typing.Union[_cdktn_78ede62e.IResolvable, ContactsSupport]],
) -> None:
    """Type checking stubs"""
    pass
