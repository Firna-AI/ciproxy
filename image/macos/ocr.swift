// SPDX-License-Identifier: Apache-2.0
// Print the text of a PNG screenshot as JSON lines: the text and its centre in
// pixels from the top left. The Setup Assistant driver checks screens with it.
import Foundation
import ImageIO
import Vision

guard CommandLine.arguments.count == 2,
    let source = CGImageSourceCreateWithURL(
        URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL, nil),
    let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
else {
    FileHandle.standardError.write("usage: ocr SCREENSHOT.png\n".data(using: .utf8)!)
    exit(2)
}
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = false
try VNImageRequestHandler(cgImage: image).perform([request])
let width = Double(image.width)
let height = Double(image.height)
for observation in request.results ?? [] {
    guard let text = observation.topCandidates(1).first?.string else { continue }
    let box = observation.boundingBox
    let line: [String: Any] = [
        "text": text,
        "x": Int(box.midX * width),
        "y": Int((1 - box.midY) * height),
        "height": Int(box.height * height),
    ]
    let data = try JSONSerialization.data(withJSONObject: line, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
}
