import AppKit
import Foundation
import Vision

let args = CommandLine.arguments.dropFirst().filter { $0 != "--" }
guard let path = args.first else {
    fputs("usage: ocr_image.swift [--] <image-path>\n", stderr)
    exit(2)
}
let url = URL(fileURLWithPath: path)

// CVE-2026-86950 (CISA KEV 2026-09-29): CoreGraphics OOB write on crafted
// files. Refuse anything that is not a JPEG SOI or PNG signature before
// NSImage / Vision see the bytes.
guard let handle = try? FileHandle(forReadingFrom: url) else { exit(0) }
let prefix = handle.readData(ofLength: 8)
try? handle.close()
let jpegSOI = prefix.count >= 3 && prefix[0] == 0xFF && prefix[1] == 0xD8 && prefix[2] == 0xFF
let pngSig: [UInt8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
let png = prefix.count >= 8 && zip(prefix, pngSig).all { $0 == $1 }
guard jpegSOI || png else { exit(0) }

guard let image = NSImage(contentsOf: url),
      let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil)
else {
    exit(0)
}

let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true

let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
do {
    try handler.perform([request])
} catch {
    fputs("vision error: \(error)\n", stderr)
    exit(1)
}

let observations = request.results ?? []
let lines: [String] = observations.compactMap { obs in
    guard let top = obs.topCandidates(1).first else { return nil }
    return top.string.trimmingCharacters(in: .whitespacesAndNewlines)
}.filter { !$0.isEmpty }

print(lines.joined(separator: "\n"))
