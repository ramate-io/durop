r'''
# `tailscale_logstream_configuration`

Refer to the OpenTofu Registry for docs: [`tailscale_logstream_configuration`](https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration).
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


class LogstreamConfiguration(
    _cdktn_78ede62e.TerraformResource,
    metaclass=jsii.JSIIMeta,
    jsii_type="tailscale.logstreamConfiguration.LogstreamConfiguration",
):
    '''Represents a {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration tailscale_logstream_configuration}.'''

    def __init__(
        self,
        scope: "_constructs_77d1e7e8.Construct",
        id: builtins.str,
        *,
        destination_type: builtins.str,
        log_type: builtins.str,
        compression_format: typing.Optional[builtins.str] = None,
        gcs_bucket: typing.Optional[builtins.str] = None,
        gcs_credentials: typing.Optional[builtins.str] = None,
        gcs_key_prefix: typing.Optional[builtins.str] = None,
        gcs_scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
        s3_access_key_id: typing.Optional[builtins.str] = None,
        s3_authentication_type: typing.Optional[builtins.str] = None,
        s3_bucket: typing.Optional[builtins.str] = None,
        s3_external_id: typing.Optional[builtins.str] = None,
        s3_key_prefix: typing.Optional[builtins.str] = None,
        s3_region: typing.Optional[builtins.str] = None,
        s3_role_arn: typing.Optional[builtins.str] = None,
        s3_secret_access_key: typing.Optional[builtins.str] = None,
        token: typing.Optional[builtins.str] = None,
        upload_period_minutes: typing.Optional[jsii.Number] = None,
        url: typing.Optional[builtins.str] = None,
        user: typing.Optional[builtins.str] = None,
        connection: typing.Optional[typing.Union[typing.Union["_cdktn_78ede62e.SSHProvisionerConnection", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.WinrmProvisionerConnection", typing.Dict[builtins.str, typing.Any]]]] = None,
        count: typing.Optional[typing.Union[jsii.Number, "_cdktn_78ede62e.TerraformCount"]] = None,
        depends_on: typing.Optional[typing.Sequence["_cdktn_78ede62e.ITerraformDependable"]] = None,
        for_each: typing.Optional["_cdktn_78ede62e.ITerraformIterator"] = None,
        lifecycle: typing.Optional[typing.Union["_cdktn_78ede62e.TerraformResourceLifecycle", typing.Dict[builtins.str, typing.Any]]] = None,
        provider: typing.Optional["_cdktn_78ede62e.TerraformProvider"] = None,
        provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union["_cdktn_78ede62e.FileProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.LocalExecProvisioner", typing.Dict[builtins.str, typing.Any]], typing.Union["_cdktn_78ede62e.RemoteExecProvisioner", typing.Dict[builtins.str, typing.Any]]]]] = None,
    ) -> None:
        '''Create a new {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration tailscale_logstream_configuration} Resource.

        :param scope: The scope in which to define this construct.
        :param id: The scoped construct ID. Must be unique amongst siblings in the same scope
        :param destination_type: The type of SIEM platform to stream to. Valid values are ``axiom``, ``cribl``, ``datadog``, ``elastic``, ``gcs``, ``panther``, ``splunk``, and ``s3``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#destination_type LogstreamConfiguration#destination_type}
        :param log_type: The type of logs to stream. Valid values are ``configuration`` (configuration audit logs) and ``network`` (network flow logs). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#log_type LogstreamConfiguration#log_type}
        :param compression_format: The compression algorithm used for logs. Valid values are ``none``, ``zstd`` or ``gzip``. Defaults to ``none``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#compression_format LogstreamConfiguration#compression_format}
        :param gcs_bucket: The name of the GCS bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_bucket LogstreamConfiguration#gcs_bucket}
        :param gcs_credentials: The encoded string of JSON that is used to authenticate for workload identity in GCS. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_credentials LogstreamConfiguration#gcs_credentials}
        :param gcs_key_prefix: The GCS key prefix for the bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_key_prefix LogstreamConfiguration#gcs_key_prefix}
        :param gcs_scopes: The GCS scopes needed to be able to write in the bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_scopes LogstreamConfiguration#gcs_scopes}
        :param s3_access_key_id: The S3 access key ID. Required if destination_type is s3 and s3_authentication_type is 'accesskey'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_access_key_id LogstreamConfiguration#s3_access_key_id}
        :param s3_authentication_type: The type of authentication to use for S3. Required if destination_type is ``s3``. Valid values are ``accesskey`` and ``rolearn``. Tailscale recommends using ``rolearn``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_authentication_type LogstreamConfiguration#s3_authentication_type}
        :param s3_bucket: The S3 bucket name. Required if destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_bucket LogstreamConfiguration#s3_bucket}
        :param s3_external_id: The AWS External ID that Tailscale supplies when authenticating using role-based authentication. Required if destination_type is 's3' and s3_authentication_type is 'rolearn'. This can be obtained via the tailscale_aws_external_id resource. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_external_id LogstreamConfiguration#s3_external_id}
        :param s3_key_prefix: An optional S3 key prefix to prepend to the auto-generated S3 key name. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_key_prefix LogstreamConfiguration#s3_key_prefix}
        :param s3_region: The region in which the S3 bucket is located. Required if destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_region LogstreamConfiguration#s3_region}
        :param s3_role_arn: ARN of the AWS IAM role that Tailscale should assume when using role-based authentication. Required if destination_type is 's3' and s3_authentication_type is 'rolearn'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_role_arn LogstreamConfiguration#s3_role_arn}
        :param s3_secret_access_key: The S3 secret access key. Required if destination_type is 's3' and s3_authentication_type is 'accesskey'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_secret_access_key LogstreamConfiguration#s3_secret_access_key}
        :param token: The token/password with which log streams to this endpoint should be authenticated, required unless destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#token LogstreamConfiguration#token}
        :param upload_period_minutes: An optional number of minutes to wait in between uploading new logs. If the quantity of logs does not fit within a single upload, multiple uploads will be made. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#upload_period_minutes LogstreamConfiguration#upload_period_minutes}
        :param url: The URL to which log streams are being posted. If destination_type is 's3' and you want to use the official Amazon S3 endpoint, leave this empty. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#url LogstreamConfiguration#url}
        :param user: The username with which log streams to this endpoint are authenticated. Only required if destination_type is 'elastic', defaults to 'user' if not set. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#user LogstreamConfiguration#user}
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__9da0328ace3cbc5e927b9c328e3297eee13714ece4ef90f0a087506aa524ba25)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument id", value=id, expected_type=type_hints["id"])
        config = LogstreamConfigurationConfig(
            destination_type=destination_type,
            log_type=log_type,
            compression_format=compression_format,
            gcs_bucket=gcs_bucket,
            gcs_credentials=gcs_credentials,
            gcs_key_prefix=gcs_key_prefix,
            gcs_scopes=gcs_scopes,
            s3_access_key_id=s3_access_key_id,
            s3_authentication_type=s3_authentication_type,
            s3_bucket=s3_bucket,
            s3_external_id=s3_external_id,
            s3_key_prefix=s3_key_prefix,
            s3_region=s3_region,
            s3_role_arn=s3_role_arn,
            s3_secret_access_key=s3_secret_access_key,
            token=token,
            upload_period_minutes=upload_period_minutes,
            url=url,
            user=user,
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
        '''Generates CDKTN code for importing a LogstreamConfiguration resource upon running "cdktn plan ".

        :param scope: The scope in which to define this construct.
        :param import_to_id: The construct id used in the generated config for the LogstreamConfiguration to import.
        :param import_from_id: The id of the existing LogstreamConfiguration that should be imported. Refer to the {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#import import section} in the documentation of this resource for the id to use
        :param provider: ? Optional instance of the provider where the LogstreamConfiguration to import is found.
        '''
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__8c52875348fd119a2f242e86108e9c619adadef9e68da3190a042b017e9fbc43)
            check_type(argname="argument scope", value=scope, expected_type=type_hints["scope"])
            check_type(argname="argument import_to_id", value=import_to_id, expected_type=type_hints["import_to_id"])
            check_type(argname="argument import_from_id", value=import_from_id, expected_type=type_hints["import_from_id"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
        return typing.cast("_cdktn_78ede62e.ImportableResource", jsii.sinvoke(cls, "generateConfigForImport", [scope, import_to_id, import_from_id, provider]))

    @jsii.member(jsii_name="resetCompressionFormat")
    def reset_compression_format(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetCompressionFormat", []))

    @jsii.member(jsii_name="resetGcsBucket")
    def reset_gcs_bucket(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetGcsBucket", []))

    @jsii.member(jsii_name="resetGcsCredentials")
    def reset_gcs_credentials(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetGcsCredentials", []))

    @jsii.member(jsii_name="resetGcsKeyPrefix")
    def reset_gcs_key_prefix(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetGcsKeyPrefix", []))

    @jsii.member(jsii_name="resetGcsScopes")
    def reset_gcs_scopes(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetGcsScopes", []))

    @jsii.member(jsii_name="resetS3AccessKeyId")
    def reset_s3_access_key_id(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3AccessKeyId", []))

    @jsii.member(jsii_name="resetS3AuthenticationType")
    def reset_s3_authentication_type(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3AuthenticationType", []))

    @jsii.member(jsii_name="resetS3Bucket")
    def reset_s3_bucket(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3Bucket", []))

    @jsii.member(jsii_name="resetS3ExternalId")
    def reset_s3_external_id(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3ExternalId", []))

    @jsii.member(jsii_name="resetS3KeyPrefix")
    def reset_s3_key_prefix(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3KeyPrefix", []))

    @jsii.member(jsii_name="resetS3Region")
    def reset_s3_region(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3Region", []))

    @jsii.member(jsii_name="resetS3RoleArn")
    def reset_s3_role_arn(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3RoleArn", []))

    @jsii.member(jsii_name="resetS3SecretAccessKey")
    def reset_s3_secret_access_key(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetS3SecretAccessKey", []))

    @jsii.member(jsii_name="resetToken")
    def reset_token(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetToken", []))

    @jsii.member(jsii_name="resetUploadPeriodMinutes")
    def reset_upload_period_minutes(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUploadPeriodMinutes", []))

    @jsii.member(jsii_name="resetUrl")
    def reset_url(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUrl", []))

    @jsii.member(jsii_name="resetUser")
    def reset_user(self) -> None:
        return typing.cast(None, jsii.invoke(self, "resetUser", []))

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
    @jsii.member(jsii_name="compressionFormatInput")
    def compression_format_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "compressionFormatInput"))

    @builtins.property
    @jsii.member(jsii_name="destinationTypeInput")
    def destination_type_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "destinationTypeInput"))

    @builtins.property
    @jsii.member(jsii_name="gcsBucketInput")
    def gcs_bucket_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "gcsBucketInput"))

    @builtins.property
    @jsii.member(jsii_name="gcsCredentialsInput")
    def gcs_credentials_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "gcsCredentialsInput"))

    @builtins.property
    @jsii.member(jsii_name="gcsKeyPrefixInput")
    def gcs_key_prefix_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "gcsKeyPrefixInput"))

    @builtins.property
    @jsii.member(jsii_name="gcsScopesInput")
    def gcs_scopes_input(self) -> typing.Optional[typing.List[builtins.str]]:
        return typing.cast(typing.Optional[typing.List[builtins.str]], jsii.get(self, "gcsScopesInput"))

    @builtins.property
    @jsii.member(jsii_name="logTypeInput")
    def log_type_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "logTypeInput"))

    @builtins.property
    @jsii.member(jsii_name="s3AccessKeyIdInput")
    def s3_access_key_id_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3AccessKeyIdInput"))

    @builtins.property
    @jsii.member(jsii_name="s3AuthenticationTypeInput")
    def s3_authentication_type_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3AuthenticationTypeInput"))

    @builtins.property
    @jsii.member(jsii_name="s3BucketInput")
    def s3_bucket_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3BucketInput"))

    @builtins.property
    @jsii.member(jsii_name="s3ExternalIdInput")
    def s3_external_id_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3ExternalIdInput"))

    @builtins.property
    @jsii.member(jsii_name="s3KeyPrefixInput")
    def s3_key_prefix_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3KeyPrefixInput"))

    @builtins.property
    @jsii.member(jsii_name="s3RegionInput")
    def s3_region_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3RegionInput"))

    @builtins.property
    @jsii.member(jsii_name="s3RoleArnInput")
    def s3_role_arn_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3RoleArnInput"))

    @builtins.property
    @jsii.member(jsii_name="s3SecretAccessKeyInput")
    def s3_secret_access_key_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "s3SecretAccessKeyInput"))

    @builtins.property
    @jsii.member(jsii_name="tokenInput")
    def token_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "tokenInput"))

    @builtins.property
    @jsii.member(jsii_name="uploadPeriodMinutesInput")
    def upload_period_minutes_input(self) -> typing.Optional[jsii.Number]:
        return typing.cast(typing.Optional[jsii.Number], jsii.get(self, "uploadPeriodMinutesInput"))

    @builtins.property
    @jsii.member(jsii_name="urlInput")
    def url_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "urlInput"))

    @builtins.property
    @jsii.member(jsii_name="userInput")
    def user_input(self) -> typing.Optional[builtins.str]:
        return typing.cast(typing.Optional[builtins.str], jsii.get(self, "userInput"))

    @builtins.property
    @jsii.member(jsii_name="compressionFormat")
    def compression_format(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "compressionFormat"))

    @compression_format.setter
    def compression_format(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__ecaf4a519ab94b845922110a4a4ce436b561902e66e1ae6fe8c6228ee1d014a8)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "compressionFormat", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="destinationType")
    def destination_type(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "destinationType"))

    @destination_type.setter
    def destination_type(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__20af0f9d49e970129519f06f04b74e652e676c9f8a4b55ae46916a7fa8d1c765)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "destinationType", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="gcsBucket")
    def gcs_bucket(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "gcsBucket"))

    @gcs_bucket.setter
    def gcs_bucket(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__74ae3f1edf92960dfc81ff77beec005fa0737cab2d0c839788bd0582a3b9e484)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "gcsBucket", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="gcsCredentials")
    def gcs_credentials(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "gcsCredentials"))

    @gcs_credentials.setter
    def gcs_credentials(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__d3a4441e3713636a09270825bb80962fd90fff4cd30bb6fdd72147d685c1c31e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "gcsCredentials", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="gcsKeyPrefix")
    def gcs_key_prefix(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "gcsKeyPrefix"))

    @gcs_key_prefix.setter
    def gcs_key_prefix(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7637b5f001979570a937db348033a107c23ebf1ee8172c2fd41b314a1c0014a4)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "gcsKeyPrefix", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="gcsScopes")
    def gcs_scopes(self) -> typing.List[builtins.str]:
        return typing.cast(typing.List[builtins.str], jsii.get(self, "gcsScopes"))

    @gcs_scopes.setter
    def gcs_scopes(self, value: typing.List[builtins.str]) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__4a0d75426f796a91d0943935d95e24527357b74ed8143b336a0a03252bb3a16a)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "gcsScopes", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="logType")
    def log_type(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "logType"))

    @log_type.setter
    def log_type(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c7ada67c68b81c1945dc8ef04d6ea92387620702aa8db2c59000dde7bdb240fe)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "logType", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3AccessKeyId")
    def s3_access_key_id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3AccessKeyId"))

    @s3_access_key_id.setter
    def s3_access_key_id(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__ffcc70561721c21211439e6fb69a2e591a29011f05dc9e531ad84e18ab165b5e)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3AccessKeyId", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3AuthenticationType")
    def s3_authentication_type(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3AuthenticationType"))

    @s3_authentication_type.setter
    def s3_authentication_type(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c485fd5b39869da8b28534c0fecef64dfef3472dc5293a7e903839c2210f1e9b)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3AuthenticationType", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3Bucket")
    def s3_bucket(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3Bucket"))

    @s3_bucket.setter
    def s3_bucket(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__c181f978cb931c48243480d992f5b78a35ae32f19a2f4c1bb9744162455efabd)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3Bucket", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3ExternalId")
    def s3_external_id(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3ExternalId"))

    @s3_external_id.setter
    def s3_external_id(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__15c877a10bb1b68341be6b799910ad19a914988a0f399494b02e874fc2390277)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3ExternalId", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3KeyPrefix")
    def s3_key_prefix(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3KeyPrefix"))

    @s3_key_prefix.setter
    def s3_key_prefix(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__fd0e71c332e3d00b83cb81b6db359aaead45fc1c7442ff541378d284951b9bde)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3KeyPrefix", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3Region")
    def s3_region(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3Region"))

    @s3_region.setter
    def s3_region(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__14d1686ef86c335e02cc166092c3c12c88ad6fae3e47e24d83423436d6c22b36)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3Region", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3RoleArn")
    def s3_role_arn(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3RoleArn"))

    @s3_role_arn.setter
    def s3_role_arn(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__8208ffedd6835f055eda83f5990634bbffcd9f6274335ed72ee343a4256a00de)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3RoleArn", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="s3SecretAccessKey")
    def s3_secret_access_key(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "s3SecretAccessKey"))

    @s3_secret_access_key.setter
    def s3_secret_access_key(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__3cf70e7aaf603f55254ff4f65b0efb795ee89c969a0f165721f413da4a90ac38)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "s3SecretAccessKey", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="token")
    def token(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "token"))

    @token.setter
    def token(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__38a45cb200c67947ac34ae8441931c1f7e975f3a9e9b3530c323d18b0a5ffe25)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "token", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="uploadPeriodMinutes")
    def upload_period_minutes(self) -> jsii.Number:
        return typing.cast(jsii.Number, jsii.get(self, "uploadPeriodMinutes"))

    @upload_period_minutes.setter
    def upload_period_minutes(self, value: jsii.Number) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__d47caf8c403178e7639c037bf35105112bca165409de4d09765b7217c6038576)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "uploadPeriodMinutes", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="url")
    def url(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "url"))

    @url.setter
    def url(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__1fc8d16873fa21c751a73d1060393d59f8fe17a93765e42962c311e5ef24bc37)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "url", value) # pyright: ignore[reportArgumentType]

    @builtins.property
    @jsii.member(jsii_name="user")
    def user(self) -> builtins.str:
        return typing.cast(builtins.str, jsii.get(self, "user"))

    @user.setter
    def user(self, value: builtins.str) -> None:
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__7121dfa1469aeeda7f2e719954245b5c6a61d404776552d0eb7050963d93475d)
            check_type(argname="argument value", value=value, expected_type=type_hints["value"])
        jsii.set(self, "user", value) # pyright: ignore[reportArgumentType]


@jsii.data_type(
    jsii_type="tailscale.logstreamConfiguration.LogstreamConfigurationConfig",
    jsii_struct_bases=[_cdktn_78ede62e.TerraformMetaArguments],
    name_mapping={
        "connection": "connection",
        "count": "count",
        "depends_on": "dependsOn",
        "for_each": "forEach",
        "lifecycle": "lifecycle",
        "provider": "provider",
        "provisioners": "provisioners",
        "destination_type": "destinationType",
        "log_type": "logType",
        "compression_format": "compressionFormat",
        "gcs_bucket": "gcsBucket",
        "gcs_credentials": "gcsCredentials",
        "gcs_key_prefix": "gcsKeyPrefix",
        "gcs_scopes": "gcsScopes",
        "s3_access_key_id": "s3AccessKeyId",
        "s3_authentication_type": "s3AuthenticationType",
        "s3_bucket": "s3Bucket",
        "s3_external_id": "s3ExternalId",
        "s3_key_prefix": "s3KeyPrefix",
        "s3_region": "s3Region",
        "s3_role_arn": "s3RoleArn",
        "s3_secret_access_key": "s3SecretAccessKey",
        "token": "token",
        "upload_period_minutes": "uploadPeriodMinutes",
        "url": "url",
        "user": "user",
    },
)
class LogstreamConfigurationConfig(_cdktn_78ede62e.TerraformMetaArguments):
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
        destination_type: builtins.str,
        log_type: builtins.str,
        compression_format: typing.Optional[builtins.str] = None,
        gcs_bucket: typing.Optional[builtins.str] = None,
        gcs_credentials: typing.Optional[builtins.str] = None,
        gcs_key_prefix: typing.Optional[builtins.str] = None,
        gcs_scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
        s3_access_key_id: typing.Optional[builtins.str] = None,
        s3_authentication_type: typing.Optional[builtins.str] = None,
        s3_bucket: typing.Optional[builtins.str] = None,
        s3_external_id: typing.Optional[builtins.str] = None,
        s3_key_prefix: typing.Optional[builtins.str] = None,
        s3_region: typing.Optional[builtins.str] = None,
        s3_role_arn: typing.Optional[builtins.str] = None,
        s3_secret_access_key: typing.Optional[builtins.str] = None,
        token: typing.Optional[builtins.str] = None,
        upload_period_minutes: typing.Optional[jsii.Number] = None,
        url: typing.Optional[builtins.str] = None,
        user: typing.Optional[builtins.str] = None,
    ) -> None:
        '''
        :param connection: 
        :param count: 
        :param depends_on: 
        :param for_each: 
        :param lifecycle: 
        :param provider: 
        :param provisioners: 
        :param destination_type: The type of SIEM platform to stream to. Valid values are ``axiom``, ``cribl``, ``datadog``, ``elastic``, ``gcs``, ``panther``, ``splunk``, and ``s3``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#destination_type LogstreamConfiguration#destination_type}
        :param log_type: The type of logs to stream. Valid values are ``configuration`` (configuration audit logs) and ``network`` (network flow logs). Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#log_type LogstreamConfiguration#log_type}
        :param compression_format: The compression algorithm used for logs. Valid values are ``none``, ``zstd`` or ``gzip``. Defaults to ``none``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#compression_format LogstreamConfiguration#compression_format}
        :param gcs_bucket: The name of the GCS bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_bucket LogstreamConfiguration#gcs_bucket}
        :param gcs_credentials: The encoded string of JSON that is used to authenticate for workload identity in GCS. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_credentials LogstreamConfiguration#gcs_credentials}
        :param gcs_key_prefix: The GCS key prefix for the bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_key_prefix LogstreamConfiguration#gcs_key_prefix}
        :param gcs_scopes: The GCS scopes needed to be able to write in the bucket. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_scopes LogstreamConfiguration#gcs_scopes}
        :param s3_access_key_id: The S3 access key ID. Required if destination_type is s3 and s3_authentication_type is 'accesskey'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_access_key_id LogstreamConfiguration#s3_access_key_id}
        :param s3_authentication_type: The type of authentication to use for S3. Required if destination_type is ``s3``. Valid values are ``accesskey`` and ``rolearn``. Tailscale recommends using ``rolearn``. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_authentication_type LogstreamConfiguration#s3_authentication_type}
        :param s3_bucket: The S3 bucket name. Required if destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_bucket LogstreamConfiguration#s3_bucket}
        :param s3_external_id: The AWS External ID that Tailscale supplies when authenticating using role-based authentication. Required if destination_type is 's3' and s3_authentication_type is 'rolearn'. This can be obtained via the tailscale_aws_external_id resource. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_external_id LogstreamConfiguration#s3_external_id}
        :param s3_key_prefix: An optional S3 key prefix to prepend to the auto-generated S3 key name. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_key_prefix LogstreamConfiguration#s3_key_prefix}
        :param s3_region: The region in which the S3 bucket is located. Required if destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_region LogstreamConfiguration#s3_region}
        :param s3_role_arn: ARN of the AWS IAM role that Tailscale should assume when using role-based authentication. Required if destination_type is 's3' and s3_authentication_type is 'rolearn'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_role_arn LogstreamConfiguration#s3_role_arn}
        :param s3_secret_access_key: The S3 secret access key. Required if destination_type is 's3' and s3_authentication_type is 'accesskey'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_secret_access_key LogstreamConfiguration#s3_secret_access_key}
        :param token: The token/password with which log streams to this endpoint should be authenticated, required unless destination_type is 's3'. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#token LogstreamConfiguration#token}
        :param upload_period_minutes: An optional number of minutes to wait in between uploading new logs. If the quantity of logs does not fit within a single upload, multiple uploads will be made. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#upload_period_minutes LogstreamConfiguration#upload_period_minutes}
        :param url: The URL to which log streams are being posted. If destination_type is 's3' and you want to use the official Amazon S3 endpoint, leave this empty. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#url LogstreamConfiguration#url}
        :param user: The username with which log streams to this endpoint are authenticated. Only required if destination_type is 'elastic', defaults to 'user' if not set. Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#user LogstreamConfiguration#user}
        '''
        if isinstance(lifecycle, dict):
            lifecycle = _cdktn_78ede62e.TerraformResourceLifecycle(**lifecycle)
        if __debug__:
            type_hints = cached_type_hints(_typecheckingstub__ded2d4b3aa171b422b3bc44663236ca9d2cab14697e48ee7f1a3bb8ce89e1b76)
            check_type(argname="argument connection", value=connection, expected_type=type_hints["connection"])
            check_type(argname="argument count", value=count, expected_type=type_hints["count"])
            check_type(argname="argument depends_on", value=depends_on, expected_type=type_hints["depends_on"])
            check_type(argname="argument for_each", value=for_each, expected_type=type_hints["for_each"])
            check_type(argname="argument lifecycle", value=lifecycle, expected_type=type_hints["lifecycle"])
            check_type(argname="argument provider", value=provider, expected_type=type_hints["provider"])
            check_type(argname="argument provisioners", value=provisioners, expected_type=type_hints["provisioners"])
            check_type(argname="argument destination_type", value=destination_type, expected_type=type_hints["destination_type"])
            check_type(argname="argument log_type", value=log_type, expected_type=type_hints["log_type"])
            check_type(argname="argument compression_format", value=compression_format, expected_type=type_hints["compression_format"])
            check_type(argname="argument gcs_bucket", value=gcs_bucket, expected_type=type_hints["gcs_bucket"])
            check_type(argname="argument gcs_credentials", value=gcs_credentials, expected_type=type_hints["gcs_credentials"])
            check_type(argname="argument gcs_key_prefix", value=gcs_key_prefix, expected_type=type_hints["gcs_key_prefix"])
            check_type(argname="argument gcs_scopes", value=gcs_scopes, expected_type=type_hints["gcs_scopes"])
            check_type(argname="argument s3_access_key_id", value=s3_access_key_id, expected_type=type_hints["s3_access_key_id"])
            check_type(argname="argument s3_authentication_type", value=s3_authentication_type, expected_type=type_hints["s3_authentication_type"])
            check_type(argname="argument s3_bucket", value=s3_bucket, expected_type=type_hints["s3_bucket"])
            check_type(argname="argument s3_external_id", value=s3_external_id, expected_type=type_hints["s3_external_id"])
            check_type(argname="argument s3_key_prefix", value=s3_key_prefix, expected_type=type_hints["s3_key_prefix"])
            check_type(argname="argument s3_region", value=s3_region, expected_type=type_hints["s3_region"])
            check_type(argname="argument s3_role_arn", value=s3_role_arn, expected_type=type_hints["s3_role_arn"])
            check_type(argname="argument s3_secret_access_key", value=s3_secret_access_key, expected_type=type_hints["s3_secret_access_key"])
            check_type(argname="argument token", value=token, expected_type=type_hints["token"])
            check_type(argname="argument upload_period_minutes", value=upload_period_minutes, expected_type=type_hints["upload_period_minutes"])
            check_type(argname="argument url", value=url, expected_type=type_hints["url"])
            check_type(argname="argument user", value=user, expected_type=type_hints["user"])
        self._values: typing.Dict[builtins.str, typing.Any] = {
            "destination_type": destination_type,
            "log_type": log_type,
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
        if compression_format is not None:
            self._values["compression_format"] = compression_format
        if gcs_bucket is not None:
            self._values["gcs_bucket"] = gcs_bucket
        if gcs_credentials is not None:
            self._values["gcs_credentials"] = gcs_credentials
        if gcs_key_prefix is not None:
            self._values["gcs_key_prefix"] = gcs_key_prefix
        if gcs_scopes is not None:
            self._values["gcs_scopes"] = gcs_scopes
        if s3_access_key_id is not None:
            self._values["s3_access_key_id"] = s3_access_key_id
        if s3_authentication_type is not None:
            self._values["s3_authentication_type"] = s3_authentication_type
        if s3_bucket is not None:
            self._values["s3_bucket"] = s3_bucket
        if s3_external_id is not None:
            self._values["s3_external_id"] = s3_external_id
        if s3_key_prefix is not None:
            self._values["s3_key_prefix"] = s3_key_prefix
        if s3_region is not None:
            self._values["s3_region"] = s3_region
        if s3_role_arn is not None:
            self._values["s3_role_arn"] = s3_role_arn
        if s3_secret_access_key is not None:
            self._values["s3_secret_access_key"] = s3_secret_access_key
        if token is not None:
            self._values["token"] = token
        if upload_period_minutes is not None:
            self._values["upload_period_minutes"] = upload_period_minutes
        if url is not None:
            self._values["url"] = url
        if user is not None:
            self._values["user"] = user

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
    def destination_type(self) -> builtins.str:
        '''The type of SIEM platform to stream to.

        Valid values are ``axiom``, ``cribl``, ``datadog``, ``elastic``, ``gcs``, ``panther``, ``splunk``, and ``s3``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#destination_type LogstreamConfiguration#destination_type}
        '''
        result = self._values.get("destination_type")
        assert result is not None, "Required property 'destination_type' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def log_type(self) -> builtins.str:
        '''The type of logs to stream. Valid values are ``configuration`` (configuration audit logs) and ``network`` (network flow logs).

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#log_type LogstreamConfiguration#log_type}
        '''
        result = self._values.get("log_type")
        assert result is not None, "Required property 'log_type' is missing"
        return typing.cast(builtins.str, result)

    @builtins.property
    def compression_format(self) -> typing.Optional[builtins.str]:
        '''The compression algorithm used for logs. Valid values are ``none``, ``zstd`` or ``gzip``. Defaults to ``none``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#compression_format LogstreamConfiguration#compression_format}
        '''
        result = self._values.get("compression_format")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def gcs_bucket(self) -> typing.Optional[builtins.str]:
        '''The name of the GCS bucket.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_bucket LogstreamConfiguration#gcs_bucket}
        '''
        result = self._values.get("gcs_bucket")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def gcs_credentials(self) -> typing.Optional[builtins.str]:
        '''The encoded string of JSON that is used to authenticate for workload identity in GCS.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_credentials LogstreamConfiguration#gcs_credentials}
        '''
        result = self._values.get("gcs_credentials")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def gcs_key_prefix(self) -> typing.Optional[builtins.str]:
        '''The GCS key prefix for the bucket.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_key_prefix LogstreamConfiguration#gcs_key_prefix}
        '''
        result = self._values.get("gcs_key_prefix")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def gcs_scopes(self) -> typing.Optional[typing.List[builtins.str]]:
        '''The GCS scopes needed to be able to write in the bucket.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#gcs_scopes LogstreamConfiguration#gcs_scopes}
        '''
        result = self._values.get("gcs_scopes")
        return typing.cast(typing.Optional[typing.List[builtins.str]], result)

    @builtins.property
    def s3_access_key_id(self) -> typing.Optional[builtins.str]:
        '''The S3 access key ID. Required if destination_type is s3 and s3_authentication_type is 'accesskey'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_access_key_id LogstreamConfiguration#s3_access_key_id}
        '''
        result = self._values.get("s3_access_key_id")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_authentication_type(self) -> typing.Optional[builtins.str]:
        '''The type of authentication to use for S3.

        Required if destination_type is ``s3``. Valid values are ``accesskey`` and ``rolearn``. Tailscale recommends using ``rolearn``.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_authentication_type LogstreamConfiguration#s3_authentication_type}
        '''
        result = self._values.get("s3_authentication_type")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_bucket(self) -> typing.Optional[builtins.str]:
        '''The S3 bucket name. Required if destination_type is 's3'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_bucket LogstreamConfiguration#s3_bucket}
        '''
        result = self._values.get("s3_bucket")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_external_id(self) -> typing.Optional[builtins.str]:
        '''The AWS External ID that Tailscale supplies when authenticating using role-based authentication.

        Required if destination_type is 's3' and s3_authentication_type is 'rolearn'. This can be obtained via the tailscale_aws_external_id resource.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_external_id LogstreamConfiguration#s3_external_id}
        '''
        result = self._values.get("s3_external_id")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_key_prefix(self) -> typing.Optional[builtins.str]:
        '''An optional S3 key prefix to prepend to the auto-generated S3 key name.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_key_prefix LogstreamConfiguration#s3_key_prefix}
        '''
        result = self._values.get("s3_key_prefix")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_region(self) -> typing.Optional[builtins.str]:
        '''The region in which the S3 bucket is located. Required if destination_type is 's3'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_region LogstreamConfiguration#s3_region}
        '''
        result = self._values.get("s3_region")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_role_arn(self) -> typing.Optional[builtins.str]:
        '''ARN of the AWS IAM role that Tailscale should assume when using role-based authentication.

        Required if destination_type is 's3' and s3_authentication_type is 'rolearn'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_role_arn LogstreamConfiguration#s3_role_arn}
        '''
        result = self._values.get("s3_role_arn")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def s3_secret_access_key(self) -> typing.Optional[builtins.str]:
        '''The S3 secret access key. Required if destination_type is 's3' and s3_authentication_type is 'accesskey'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#s3_secret_access_key LogstreamConfiguration#s3_secret_access_key}
        '''
        result = self._values.get("s3_secret_access_key")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def token(self) -> typing.Optional[builtins.str]:
        '''The token/password with which log streams to this endpoint should be authenticated, required unless destination_type is 's3'.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#token LogstreamConfiguration#token}
        '''
        result = self._values.get("token")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def upload_period_minutes(self) -> typing.Optional[jsii.Number]:
        '''An optional number of minutes to wait in between uploading new logs.

        If the quantity of logs does not fit within a single upload, multiple uploads will be made.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#upload_period_minutes LogstreamConfiguration#upload_period_minutes}
        '''
        result = self._values.get("upload_period_minutes")
        return typing.cast(typing.Optional[jsii.Number], result)

    @builtins.property
    def url(self) -> typing.Optional[builtins.str]:
        '''The URL to which log streams are being posted.

        If destination_type is 's3' and you want to use the official Amazon S3 endpoint, leave this empty.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#url LogstreamConfiguration#url}
        '''
        result = self._values.get("url")
        return typing.cast(typing.Optional[builtins.str], result)

    @builtins.property
    def user(self) -> typing.Optional[builtins.str]:
        '''The username with which log streams to this endpoint are authenticated.

        Only required if destination_type is 'elastic', defaults to 'user' if not set.

        Docs at OpenTofu Registry: {@link https://search.opentofu.org/provider/tailscale/tailscale/v0.29.2/docs/resources/logstream_configuration#user LogstreamConfiguration#user}
        '''
        result = self._values.get("user")
        return typing.cast(typing.Optional[builtins.str], result)

    def __eq__(self, rhs: typing.Any) -> builtins.bool:
        return isinstance(rhs, self.__class__) and rhs._values == self._values

    def __ne__(self, rhs: typing.Any) -> builtins.bool:
        return not (rhs == self)

    def __repr__(self) -> str:
        return "LogstreamConfigurationConfig(%s)" % ", ".join(
            k + "=" + repr(v) for k, v in self._values.items()
        )


__all__ = [
    "LogstreamConfiguration",
    "LogstreamConfigurationConfig",
]

publication.publish()

def _typecheckingstub__9da0328ace3cbc5e927b9c328e3297eee13714ece4ef90f0a087506aa524ba25(
    scope: _constructs_77d1e7e8.Construct,
    id: builtins.str,
    *,
    destination_type: builtins.str,
    log_type: builtins.str,
    compression_format: typing.Optional[builtins.str] = None,
    gcs_bucket: typing.Optional[builtins.str] = None,
    gcs_credentials: typing.Optional[builtins.str] = None,
    gcs_key_prefix: typing.Optional[builtins.str] = None,
    gcs_scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
    s3_access_key_id: typing.Optional[builtins.str] = None,
    s3_authentication_type: typing.Optional[builtins.str] = None,
    s3_bucket: typing.Optional[builtins.str] = None,
    s3_external_id: typing.Optional[builtins.str] = None,
    s3_key_prefix: typing.Optional[builtins.str] = None,
    s3_region: typing.Optional[builtins.str] = None,
    s3_role_arn: typing.Optional[builtins.str] = None,
    s3_secret_access_key: typing.Optional[builtins.str] = None,
    token: typing.Optional[builtins.str] = None,
    upload_period_minutes: typing.Optional[jsii.Number] = None,
    url: typing.Optional[builtins.str] = None,
    user: typing.Optional[builtins.str] = None,
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

def _typecheckingstub__8c52875348fd119a2f242e86108e9c619adadef9e68da3190a042b017e9fbc43(
    scope: _constructs_77d1e7e8.Construct,
    import_to_id: builtins.str,
    import_from_id: builtins.str,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__ecaf4a519ab94b845922110a4a4ce436b561902e66e1ae6fe8c6228ee1d014a8(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__20af0f9d49e970129519f06f04b74e652e676c9f8a4b55ae46916a7fa8d1c765(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__74ae3f1edf92960dfc81ff77beec005fa0737cab2d0c839788bd0582a3b9e484(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__d3a4441e3713636a09270825bb80962fd90fff4cd30bb6fdd72147d685c1c31e(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7637b5f001979570a937db348033a107c23ebf1ee8172c2fd41b314a1c0014a4(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__4a0d75426f796a91d0943935d95e24527357b74ed8143b336a0a03252bb3a16a(
    value: typing.List[builtins.str],
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c7ada67c68b81c1945dc8ef04d6ea92387620702aa8db2c59000dde7bdb240fe(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__ffcc70561721c21211439e6fb69a2e591a29011f05dc9e531ad84e18ab165b5e(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c485fd5b39869da8b28534c0fecef64dfef3472dc5293a7e903839c2210f1e9b(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__c181f978cb931c48243480d992f5b78a35ae32f19a2f4c1bb9744162455efabd(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__15c877a10bb1b68341be6b799910ad19a914988a0f399494b02e874fc2390277(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__fd0e71c332e3d00b83cb81b6db359aaead45fc1c7442ff541378d284951b9bde(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__14d1686ef86c335e02cc166092c3c12c88ad6fae3e47e24d83423436d6c22b36(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__8208ffedd6835f055eda83f5990634bbffcd9f6274335ed72ee343a4256a00de(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__3cf70e7aaf603f55254ff4f65b0efb795ee89c969a0f165721f413da4a90ac38(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__38a45cb200c67947ac34ae8441931c1f7e975f3a9e9b3530c323d18b0a5ffe25(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__d47caf8c403178e7639c037bf35105112bca165409de4d09765b7217c6038576(
    value: jsii.Number,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__1fc8d16873fa21c751a73d1060393d59f8fe17a93765e42962c311e5ef24bc37(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__7121dfa1469aeeda7f2e719954245b5c6a61d404776552d0eb7050963d93475d(
    value: builtins.str,
) -> None:
    """Type checking stubs"""
    pass

def _typecheckingstub__ded2d4b3aa171b422b3bc44663236ca9d2cab14697e48ee7f1a3bb8ce89e1b76(
    *,
    connection: typing.Optional[typing.Union[typing.Union[_cdktn_78ede62e.SSHProvisionerConnection, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.WinrmProvisionerConnection, typing.Dict[builtins.str, typing.Any]]]] = None,
    count: typing.Optional[typing.Union[jsii.Number, _cdktn_78ede62e.TerraformCount]] = None,
    depends_on: typing.Optional[typing.Sequence[_cdktn_78ede62e.ITerraformDependable]] = None,
    for_each: typing.Optional[_cdktn_78ede62e.ITerraformIterator] = None,
    lifecycle: typing.Optional[typing.Union[_cdktn_78ede62e.TerraformResourceLifecycle, typing.Dict[builtins.str, typing.Any]]] = None,
    provider: typing.Optional[_cdktn_78ede62e.TerraformProvider] = None,
    provisioners: typing.Optional[typing.Sequence[typing.Union[typing.Union[_cdktn_78ede62e.FileProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.LocalExecProvisioner, typing.Dict[builtins.str, typing.Any]], typing.Union[_cdktn_78ede62e.RemoteExecProvisioner, typing.Dict[builtins.str, typing.Any]]]]] = None,
    destination_type: builtins.str,
    log_type: builtins.str,
    compression_format: typing.Optional[builtins.str] = None,
    gcs_bucket: typing.Optional[builtins.str] = None,
    gcs_credentials: typing.Optional[builtins.str] = None,
    gcs_key_prefix: typing.Optional[builtins.str] = None,
    gcs_scopes: typing.Optional[typing.Sequence[builtins.str]] = None,
    s3_access_key_id: typing.Optional[builtins.str] = None,
    s3_authentication_type: typing.Optional[builtins.str] = None,
    s3_bucket: typing.Optional[builtins.str] = None,
    s3_external_id: typing.Optional[builtins.str] = None,
    s3_key_prefix: typing.Optional[builtins.str] = None,
    s3_region: typing.Optional[builtins.str] = None,
    s3_role_arn: typing.Optional[builtins.str] = None,
    s3_secret_access_key: typing.Optional[builtins.str] = None,
    token: typing.Optional[builtins.str] = None,
    upload_period_minutes: typing.Optional[jsii.Number] = None,
    url: typing.Optional[builtins.str] = None,
    user: typing.Optional[builtins.str] = None,
) -> None:
    """Type checking stubs"""
    pass
