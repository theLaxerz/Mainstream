import AppKit
import Foundation
import Vision

guard CommandLine.arguments.count > 1 else {
    fputs("usage: ocr_image.swift <image-path>\n", stderr)
    exit(2)
}

let path = CommandLine.arguments[1]
let url = URL(fileURLWithPath: path)

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
