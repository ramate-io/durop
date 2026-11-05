// Read-only authored-fixture conversion check; not a Bevy presentation proof.
import AVFoundation
import CoreImage
import Foundation
guard CommandLine.arguments.count==2 else {fputs("usage: check-orientation.swift PATH\n",stderr);exit(2)}
do {
let url=URL(fileURLWithPath:CommandLine.arguments[1])
let asset=AVURLAsset(url:url)
let reader=try AVAssetReader(asset:asset)
let output=AVAssetReaderTrackOutput(track:asset.tracks(withMediaType:.video)[0],outputSettings:[kCVPixelBufferPixelFormatTypeKey as String:kCVPixelFormatType_32BGRA])
reader.add(output)
guard reader.startReading(),let sample=output.copyNextSampleBuffer(),let buffer=CMSampleBufferGetImageBuffer(sample) else {fputs("fixture reader failed\n",stderr);exit(1)}
let image=CIImage(cvPixelBuffer:buffer)
var bytes=[UInt8](repeating:0,count:128*128*4)
let context=CIContext()
bytes.withUnsafeMutableBytes{context.render(image,toBitmap:$0.baseAddress!,rowBytes:128*4,bounds:image.extent,format:.RGBA8,colorSpace:CGColorSpace(name:CGColorSpace.sRGB)!)}
func pixel(_ x:Int,_ y:Int)->[UInt8]{let i=(y*128+x)*4;return Array(bytes[i..<i+4])}
let green=pixel(20,20),blue=pixel(100,100),edge=pixel(17,60)
print("top-left authored raster RGBA:",green,"bottom-right:",blue,"partial-alpha edge:",edge)
guard green[1]>200 && Int(green[1])>Int(green[0])*2 && blue[2]>200 && blue[1]<80 && (100...160).contains(Int(edge[3])) else {exit(1)}

} catch {fputs("fixture conversion failed: \(error)\n",stderr);exit(1)}
