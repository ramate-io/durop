"""The only deployment authoring source: typed CDK Terrain constructs."""
import json
from cdktn import App, LocalBackend, TerraformResourceLifecycle, TerraformStack
from constructs import Construct
from imports.tailscale.acl import Acl
from imports.tailscale.dns_preferences import DnsPreferences
from imports.tailscale.provider import TailscaleProvider
from settings import Settings


class DuropNetwork(TerraformStack):
    def __init__(self, scope: Construct, settings: Settings):
        super().__init__(scope, 'durop-network')
        LocalBackend(self, path=str(settings.state_dir / 'terraform.tfstate'))
        TailscaleProvider(self, 'tailscale')
        policy = Acl(self, 'policy', acl=json.dumps(settings.policy, sort_keys=True, separators=(',', ':')),
                     overwrite_existing_content=False, reset_acl_on_destroy=False,
                     lifecycle=TerraformResourceLifecycle(prevent_destroy=True))
        policy.override_logical_id('policy')
        if settings.magic_dns is not None:
            dns = DnsPreferences(self, 'dns', magic_dns=settings.magic_dns,
                                 lifecycle=TerraformResourceLifecycle(prevent_destroy=True))
            dns.override_logical_id('dns')


def synthesize(settings: Settings, output: str) -> None:
    app = App(outdir=output, context={'cdktn:targetVersions': {'opentofu': '>=1.8.0'}})
    DuropNetwork(app, settings)
    app.synth()
