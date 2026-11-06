#!/usr/bin/env python3
"""Rebuild or check the curated pack; sources and tools are never used at playback."""

import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parent
# Bound acquisition separately from the size of the distributed derived pack.
MAX_SOURCE_BYTES = 32 * 1024 * 1024
MAX_PACK_BYTES = 50 * 1024 * 1024


def sha256(path):
    """Hash bytes rather than trusting transport filenames or mutable source URLs."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(ffmpeg, *args):
    """Bound each conversion and fail on malformed media instead of writing placeholders."""
    subprocess.run([ffmpeg, '-hide_banner', '-loglevel', 'error', '-y', *map(str, args)],
                   check=True, timeout=90)


def build(args, entries):
    """Acquire pinned short sources, derive compact clips and capture real thumbnails."""
    args.cache.mkdir(parents=True, exist_ok=True)
    for entry in entries:
        source = args.cache / entry['source_sha256']
        if not source.exists():
            if not args.download:
                raise ValueError(f"missing cached source {entry['id']}; use --download explicitly")
            request = urllib.request.Request(entry['source_url'], headers={'User-Agent': 'DuropPackBuilder/1.0'})
            with urllib.request.urlopen(request, timeout=30) as response:
                data = response.read(MAX_SOURCE_BYTES + 1)
            if len(data) > MAX_SOURCE_BYTES:
                raise ValueError('source exceeds short-asset download limit')
            if hashlib.sha256(data).hexdigest() != entry['source_sha256']:
                raise ValueError(f"source changed: {entry['id']}")
            source.write_bytes(data)
        if sha256(source) != entry['source_sha256']:
            raise ValueError(f"cached source changed: {entry['id']}")
        base = ROOT / entry['category'] / entry['id']
        base.parent.mkdir(exist_ok=True)
        output = base.with_suffix('.' + entry['format'])
        common = ['-stream_loop', '-1', '-ss', entry.get('start_seconds', 0),
                  '-t', entry['duration_seconds'], '-i', source, '-map_metadata', '-1']
        if entry['format'] == 'gif':
            run(args.ffmpeg, *common, '-an', '-filter_complex',
                'fps=12,scale=320:-2:flags=lanczos,split[a][b];'
                '[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=sierra2_4a',
                '-loop', '0', output)
        elif entry['format'] == 'mp4':
            run(args.ffmpeg, *common, '-vf', 'scale=480:-2:flags=lanczos',
                '-c:v', 'libx264', '-crf', '26', '-pix_fmt', 'yuv420p',
                '-c:a', 'aac', '-b:a', '96k', '-movflags', '+faststart', output)
        else:
            run(args.ffmpeg, *common, '-vn', '-c:a', 'libmp3lame', '-b:a', '96k', output)
        if entry['format'] == 'mp3':
            run(args.ffmpeg, '-i', output, '-filter_complex',
                'aformat=channel_layouts=mono,showwavespic=s=320x180:colors=0x45d6c8',
                '-frames:v', 1, base.with_suffix('.png'))
        else:
            run(args.ffmpeg, '-ss', min(entry['duration_seconds'] / 2, 1), '-i', output,
                '-vf', 'scale=320:-2', '-frames:v', 1, base.with_suffix('.png'))
        metadata = dict(id=entry['id'], title=entry['title'], asset=output.name,
                        tags=entry['tags'], playback=dict(retrigger='restart', loop=False,
                        maxDurationMs=int(entry['duration_seconds'] * 1000 + 300)))
        if entry['format'] != 'mp3':
            metadata['visual'] = dict(anchor='bottom-right', widthFraction=0.32, fit='contain')
        base.with_suffix('.json').write_text(json.dumps(metadata, indent=2) + '\n')
    paths = [ROOT / 'durop.json']
    for entry in entries:
        base = ROOT / entry['category'] / entry['id']
        paths.extend(base.with_suffix(ext) for ext in ('.json', '.png', '.' + entry['format']))
    (ROOT / 'SHA256SUMS').write_text(''.join(f'{sha256(p)}  {p.relative_to(ROOT)}\n' for p in sorted(paths)))


def verify(args, entries):
    """Check independent clips, provenance, containment, thumbnails and bounded decodes."""
    assert len(entries) == 31, 'expected 31 curated assets'
    assert len({entry['id'] for entry in entries}) == 31
    assert collections.Counter(e['format'] for e in entries) == {'gif': 12, 'mp3': 12, 'mp4': 7}
    hashes = set()
    expected = set()
    for line in (ROOT / 'SHA256SUMS').read_text().splitlines():
        digest, name = line.split('  ', 1)
        path = ROOT / name
        assert path.resolve().is_relative_to(ROOT.resolve()), name
        assert not path.is_symlink() and path.is_file(), name
        assert sha256(path) == digest, name
        expected.add(path)
    assert len(expected) == 94, 'manifest plus three files per clip must be hash-pinned'
    assert sum(p.stat().st_size for p in ROOT.rglob('*') if p.is_file()) < MAX_PACK_BYTES
    for entry in entries:
        for key in ('source_url', 'source_sha256', 'author', 'license', 'license_url', 'source_page', 'changes'):
            assert entry[key], (entry['id'], key)
        assert re.fullmatch(r'[0-9a-f]{64}', entry['source_sha256'])
        assert 1 <= entry['duration_seconds'] <= 8
        base = ROOT / entry['category'] / entry['id']
        media = base.with_suffix('.' + entry['format'])
        assert all(base.with_suffix(ext) in expected for ext in ('.json', '.png', '.' + entry['format']))
        assert media in expected and sha256(media) not in hashes
        hashes.add(sha256(media))
        data = base.with_suffix('.png').read_bytes()
        assert len(data) <= 1024 * 1024
        assert data[:8] == b'\x89PNG\r\n\x1a\n'
        width, height = struct.unpack('>II', data[16:24])
        assert 0 < width <= 512 and 0 < height <= 512
        clip = json.loads(base.with_suffix('.json').read_text())
        assert clip['id'] == entry['id'] and len(clip['tags']) >= 2
        # A complete decode catches truncated/corrupt containers, not just headers.
        if not args.metadata_only:
            run(args.ffmpeg, '-i', media, '-f', 'null', '-')
            # Sound-bearing videos must retain a decodable audio stream after conversion.
            if entry['format'] == 'mp4' and entry.get('silent') is False:
                run(args.ffmpeg, '-i', media, '-map', '0:a:0', '-f', 'null', '-')
    actual_metadata = set(ROOT.glob('*/*.json'))
    assert actual_metadata == {p for p in expected if p.suffix == '.json' and p.name != 'durop.json'}
    print('Verified 31 unique media clips:', dict(collections.Counter(e['format'] for e in entries)))


def main():
    """Require explicit acquisition; ordinary verification is fully offline."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['build', 'verify'])
    parser.add_argument('--ffmpeg', default=os.environ.get('FFMPEG', 'ffmpeg'))
    parser.add_argument('--cache', type=Path, default=Path('/tmp/durop-reaction-sources'))
    parser.add_argument('--download', action='store_true')
    parser.add_argument('--metadata-only', action='store_true',
                        help='verify hashes, PNG bounds and provenance without FFmpeg (CI)')
    args = parser.parse_args()
    entries = json.loads((ROOT / 'SOURCES.lock').read_text())
    if args.command == 'build':
        build(args, entries)
    verify(args, entries)


if __name__ == '__main__':
    main()
