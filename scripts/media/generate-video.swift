// Independently authored native encoder fixture. Run after generate.py.
import AVFoundation
import CoreVideo
import Foundation

let root = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
func generate(_ name: String, codec: AVVideoCodecType, alpha: Bool, orientation: Bool = false) throws {
    let url = root.appendingPathComponent(name)
    try? FileManager.default.removeItem(at: url)
    let writer = try AVAssetWriter(outputURL: url, fileType: .mov)
    let input = AVAssetWriterInput(mediaType: .video, outputSettings: [
        AVVideoCodecKey: codec, AVVideoWidthKey: 128, AVVideoHeightKey: 128
    ])
    let adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
        kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
        kCVPixelBufferWidthKey as String: 128, kCVPixelBufferHeightKey as String: 128,
        kCVPixelBufferIOSurfacePropertiesKey as String: [:]
    ])
    writer.add(input)
    guard writer.startWriting() else { throw writer.error! }
    writer.startSession(atSourceTime: .zero)
    for frame in 0..<60 {
        let deadline = Date().addingTimeInterval(5)
        while !input.isReadyForMoreMediaData && Date() < deadline { Thread.sleep(forTimeInterval: 0.001) }
        guard input.isReadyForMoreMediaData else { throw NSError(domain: "fixture encoder timeout", code: 1) }
        var pixel: CVPixelBuffer?
        CVPixelBufferPoolCreatePixelBuffer(nil, adaptor.pixelBufferPool!, &pixel)
        let buffer = pixel!
        CVPixelBufferLockBaseAddress(buffer, [])
        let stride = CVPixelBufferGetBytesPerRow(buffer)
        let bytes = CVPixelBufferGetBaseAddress(buffer)!.assumingMemoryBound(to: UInt8.self)
        let flash = (15..<18).contains(frame) || (45..<48).contains(frame)
        for y in 0..<128 { for x in 0..<128 {
            let offset = y * stride + x * 4
            let ring = (16..<112).contains(x) && (16..<112).contains(y) && !((40..<88).contains(x) && (40..<88).contains(y))
            let visible = !alpha || ring
            let value: UInt8 = flash ? 255 : 40
            bytes[offset] = visible ? value : 0
            bytes[offset + 1] = visible && flash ? value : 0
            bytes[offset + 2] = visible ? value : 0
            bytes[offset + 3] = visible ? 255 : 0
            if orientation {
                // Authored raster row zero is the top. Unequal markers expose
                // vertical mirroring; partial-alpha outer edges expose fringes.
                if (16..<32).contains(x) && (16..<40).contains(y) {
                    bytes[offset] = 0; bytes[offset + 1] = 255; bytes[offset + 2] = 0; bytes[offset + 3] = 255
                } else if (88..<112).contains(x) && (96..<112).contains(y) {
                    bytes[offset] = 255; bytes[offset + 1] = 0; bytes[offset + 2] = 0; bytes[offset + 3] = 255
                } else if ring && (x < 19 || x >= 109 || y < 19 || y >= 109) {
                    bytes[offset] /= 2; bytes[offset + 1] /= 2; bytes[offset + 2] /= 2; bytes[offset + 3] = 128
                }
            }
        } }
        CVPixelBufferUnlockBaseAddress(buffer, [])
        guard adaptor.append(buffer, withPresentationTime: CMTime(value: Int64(frame), timescale: 30)) else { throw writer.error! }
    }
    writer.endSession(atSourceTime: CMTime(value: 2, timescale: 1))
    input.markAsFinished()
    let done = DispatchSemaphore(value: 0)
    writer.finishWriting { done.signal() }
    guard done.wait(timeout: .now() + 10) == .success, writer.status == .completed else {
        throw writer.error ?? NSError(domain: "fixture writer timeout", code: 1)
    }
}
do {
if CommandLine.arguments.contains("--orientation-only") {
    try generate("orientation-alpha.mov", codec: .hevcWithAlpha, alpha: true, orientation: true)
    print("Generated asymmetric orientation-alpha.mov")
    exit(0)
}
try generate("orientation-alpha.mov", codec: .hevcWithAlpha, alpha: true, orientation: true)
try generate("silent-h264.mov", codec: .h264, alpha: false)
try generate("alpha-hevc.mov", codec: .hevcWithAlpha, alpha: true)
let composition = AVMutableComposition()
let video = AVURLAsset(url: root.appendingPathComponent("silent-h264.mov"))
let audio = AVURLAsset(url: root.appendingPathComponent("flash-beep.wav"))
let range = CMTimeRange(start: .zero, duration: CMTime(value: 2, timescale: 1))
try composition.addMutableTrack(withMediaType: .video, preferredTrackID: kCMPersistentTrackID_Invalid)!.insertTimeRange(range, of: video.tracks(withMediaType: .video)[0], at: .zero)
try composition.addMutableTrack(withMediaType: .audio, preferredTrackID: kCMPersistentTrackID_Invalid)!.insertTimeRange(range, of: audio.tracks(withMediaType: .audio)[0], at: .zero)
let export = AVAssetExportSession(asset: composition, presetName: AVAssetExportPresetMediumQuality)!
let output = root.appendingPathComponent("flash-beep.mp4")
try? FileManager.default.removeItem(at: output)
export.outputURL = output
export.outputFileType = .mp4
let done = DispatchSemaphore(value: 0)
export.exportAsynchronously { done.signal() }
guard done.wait(timeout: .now() + 10) == .success, export.status == .completed else { throw export.error ?? NSError(domain: "fixture mux timeout", code: 1) }
print("Generated H264/AAC flash-beep.mp4 and transparent HEVC alpha-hevc.mov")

} catch {
    fputs("fixture generation failed: \(error)\n", stderr)
    exit(1)
}
