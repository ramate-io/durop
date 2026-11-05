#!/usr/bin/env python3
"""Isolated native HTTPS/credential-helper acceptance; no account credentials used."""
import base64
import http.server
import os
from pathlib import Path
import shutil
import ssl
import subprocess
import tempfile
import threading
import urllib.parse


def run(args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


project = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix="durop-https-") as temporary:
    root = Path(temporary)
    pack = root / "pack"
    shutil.copytree(project / "examples/sample-pack", pack)
    marker = root / "forbidden-execution"
    forbidden = root / "forbidden"
    forbidden.write_text(f'#!/bin/sh\ntouch "{marker}"\nexit 1\n')
    forbidden.chmod(0o700)
    hooks = root / "hooks"
    hooks.mkdir()
    shutil.copy(forbidden, hooks / "reference-transaction")
    helper_log = root / "helper-operations"
    helper = root / "credential-helper"
    helper.write_text(
        '#!/bin/sh\n'
        'cat >/dev/null\n'
        f'printf "%s\\n" "$1" >> "{helper_log}"\n'
        'if [ "$1" = get ]; then\n'
        '  printf "username=fixture\\npassword=fixture-only\\n\\n"\n'
        'fi\n'
    )
    helper.chmod(0o700)
    config = root / "gitconfig"
    config.write_text(
        f'[credential]\n\thelper =\n\thelper = {helper}\n'
        f'[core]\n\thooksPath = {hooks}\n'
        f'[filter "forbidden"]\n\tclean = cat\n\tsmudge = {forbidden}\n\trequired = true\n'
    )
    env = os.environ.copy()
    # Isolate trusted test configuration without editing the user's Git setup.
    for key in list(env):
        if key.startswith("GIT_CONFIG_") or key in ("GIT_DIR", "GIT_WORK_TREE"):
            del env[key]
    env.update(GIT_CONFIG_GLOBAL=str(config), GIT_CONFIG_NOSYSTEM="1")
    git = ["git", "-c", "core.hooksPath=/dev/null", "-C", str(pack)]
    run(git + ["init", "--template=/dev/null"], env=env, capture_output=True)
    (pack / ".gitattributes").write_text("*.wav filter=forbidden\n")
    shutil.copy(forbidden, pack / "do-not-run.sh")
    run(git + ["add", "--all"], env=env, capture_output=True)
    run(git + ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
               "commit", "-m", "HTTPS fixture"], env=env, capture_output=True)

    certificate_config = root / "certificate.cnf"
    certificate_config.write_text(
        "[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n"
        "[dn]\nCN=localhost\n[ext]\nsubjectAltName=DNS:localhost\n"
        "basicConstraints=critical,CA:TRUE\n"
    )
    certificate = root / "certificate.pem"
    key = root / "key.pem"
    run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
         "-config", str(certificate_config), "-keyout", str(key), "-out", str(certificate)],
        capture_output=True)
    backend = Path(subprocess.check_output(["git", "--exec-path"], env=env, text=True).strip()) / "git-http-backend"

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def request(self):
            if self.headers.get("Authorization") != "Basic " + base64.b64encode(b"fixture:fixture-only").decode():
                self.send_response(401)
                self.send_header("WWW-Authenticate", 'Basic realm="catalog fixture"')
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            parsed = urllib.parse.urlsplit(self.path)
            body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
            backend_env = dict(env, GIT_PROJECT_ROOT=str(root), GIT_HTTP_EXPORT_ALL="1",
                               PATH_INFO=parsed.path, QUERY_STRING=parsed.query,
                               REQUEST_METHOD=self.command, CONTENT_TYPE=self.headers.get("Content-Type", ""),
                               CONTENT_LENGTH=str(len(body)), REMOTE_USER="fixture")
            if self.headers.get("Git-Protocol"):
                backend_env["HTTP_GIT_PROTOCOL"] = self.headers["Git-Protocol"]
            result = subprocess.run([str(backend)], env=backend_env, input=body, capture_output=True, check=True)
            header, payload = result.stdout.split(b"\r\n\r\n", 1)
            fields = [line.decode().split(":", 1) for line in header.split(b"\r\n")]
            status = next((int(value.strip().split()[0]) for name, value in fields if name.lower() == "status"), 200)
            self.send_response(status)
            for name, value in fields:
                if name.lower() != "status":
                    self.send_header(name, value.strip())
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        do_GET = request
        do_POST = request

    server = http.server.ThreadingHTTPServer(("localhost", 0), Handler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(certificate, key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    env.update(GIT_SSL_CAINFO=str(certificate), DUROP_TEST_REPOSITORY=str(pack),
               DUROP_TEST_HTTPS_URL=f"https://localhost:{server.server_port}/pack/.git")
    try:
        run(["cargo", "test", "-p", "durop-catalog", "--locked", "cache_tests::https_authenticated_cache_acceptance",
             "--", "--ignored", "--exact"], cwd=project, env=env)
        assert not marker.exists(), "a repository script, hook or smudge filter executed"
        assert "get" in helper_log.read_text().splitlines(), "credential helper was not used"
        print("HTTPS auth/cache acceptance passed; helper used; scripts/hooks/filters did not execute.")
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
