// Native AVFoundation decoding/clock probe; independently authored for Durop.
// Run: swift scripts/media/probe.swift PATH [--audible]
import AVFoundation
import CoreVideo
import Foundation

let arguments = CommandLine.arguments.dropFirst()
guard let path = arguments.first else { fputs("usage: probe.swift PATH [--audible]\n", stderr); exit(2) }
let url = URL(fileURLWithPath: path)
let audible = arguments.contains("--audible")
let started = ProcessInfo.processInfo.systemUptime
let asset = AVURLAsset(url: url)
let item = AVPlayerItem(asset: asset)
let output = AVPlayerItemVideoOutput(pixelBufferAttributes: [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
    kCVPixelBufferIOSurfacePropertiesKey as String: [:]
])
item.add(output)
let player = AVPlayer(playerItem: item)
player.isMuted = !audible
func pump(_ seconds: Double) {
    RunLoop.current.run(until: Date().addingTimeInterval(seconds))
}
while item.status == .unknown && ProcessInfo.processInfo.systemUptime - started < 8 {
    pump(0.005)
}
guard item.status == .readyToPlay else {
    let message = item.error?.localizedDescription ?? "readiness timeout"
    print("{\"error\":\"\(message.replacingOccurrences(of: "\"", with: "'"))\"}")
    exit(1)
}
let readyMS = (ProcessInfo.processInfo.systemUptime - started) * 1000
let duration = CMTimeGetSeconds(item.duration)
var frames = 0
var firstFrameMS: Double? = nil
var dimensions = [0, 0]
var strides = Set<Int>()
var alphaMin: UInt8 = 255
var alphaMax: UInt8 = 0
var presentation: Double = 0
player.play()
let playing = ProcessInfo.processInfo.systemUptime
while ProcessInfo.processInfo.systemUptime - playing < min(4, duration + 0.2) {
    let time = player.currentTime()
    if output.hasNewPixelBuffer(forItemTime: time) {
        var display = CMTime.invalid
        if let buffer = output.copyPixelBuffer(forItemTime: time, itemTimeForDisplay: &display) {
            CVPixelBufferLockBaseAddress(buffer, .readOnly)
            let width = CVPixelBufferGetWidth(buffer)
            let height = CVPixelBufferGetHeight(buffer)
            let stride = CVPixelBufferGetBytesPerRow(buffer)
            dimensions = [width, height]
            strides.insert(stride)
            if let base = CVPixelBufferGetBaseAddress(buffer)?.assumingMemoryBound(to: UInt8.self) {
                for row in 0..<height {
                    for column in 0..<width {
                        let alpha = base[row * stride + column * 4 + 3]
                        alphaMin = min(alphaMin, alpha)
                        alphaMax = max(alphaMax, alpha)
                    }
                }
            }
            CVPixelBufferUnlockBaseAddress(buffer, .readOnly)
            frames += 1
            presentation = CMTimeGetSeconds(display)
            if firstFrameMS == nil { firstFrameMS = (ProcessInfo.processInfo.systemUptime - playing) * 1000 }
        }
    }
    if item.status == .failed { break }
    pump(0.005)
}
let position = CMTimeGetSeconds(player.currentTime())
let stopping = ProcessInfo.processInfo.systemUptime
player.pause()
player.replaceCurrentItem(with: nil)
let stopMS = (ProcessInfo.processInfo.systemUptime - stopping) * 1000
let tracks = asset.tracks.map { track in
    ["type": track.mediaType.rawValue,
     "codecs": track.formatDescriptions.map { description in
        let code = CMFormatDescriptionGetMediaSubType(description as! CMFormatDescription)
        return String(bytes: [UInt8((code >> 24) & 255), UInt8((code >> 16) & 255), UInt8((code >> 8) & 255), UInt8(code & 255)], encoding: .ascii) ?? "unknown"
     }] as [String: Any]
}
let result: [String: Any] = [
    "file": url.lastPathComponent, "ready_ms": readyMS, "duration_seconds": duration,
    "position_seconds": position, "frames": frames, "first_frame_ms": firstFrameMS as Any? ?? NSNull(),
    "last_presentation_seconds": presentation, "dimensions": dimensions, "row_strides": strides.sorted(),
    "alpha_range": frames > 0 ? [alphaMin, alphaMax] as Any : NSNull(), "stop_ms": stopMS, "tracks": tracks,
    "audio_muted": !audible, "status": item.status.rawValue
]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
