import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

struct Bounds {
    var minX: Int
    var minY: Int
    var maxX: Int
    var maxY: Int

    var width: Int { maxX - minX + 1 }
    var height: Int { maxY - minY + 1 }

    mutating func include(x: Int, y: Int) {
        minX = min(minX, x)
        minY = min(minY, y)
        maxX = max(maxX, x)
        maxY = max(maxY, y)
    }
}

struct Component {
    let id: Int32
    var count: Int
    var bounds: Bounds
}

let projectRoot = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
let sourceURL = projectRoot.deletingLastPathComponent().appendingPathComponent("Kotha.jpeg")
let publicBrandURL = projectRoot.appendingPathComponent("public/brand")
let iconURL = projectRoot.appendingPathComponent("src-tauri/icons")
let trayURL = projectRoot.appendingPathComponent("src-tauri/resources")

try FileManager.default.createDirectory(at: publicBrandURL, withIntermediateDirectories: true)

guard
    let imageSource = CGImageSourceCreateWithURL(sourceURL as CFURL, nil),
    let sourceImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil)
else {
    fatalError("Could not load \(sourceURL.path)")
}

let sourceWidth = sourceImage.width
let sourceHeight = sourceImage.height
let colorSpace = CGColorSpaceCreateDeviceRGB()
let bitmapInfo = CGBitmapInfo.byteOrder32Big.rawValue | CGImageAlphaInfo.premultipliedLast.rawValue

func context(width: Int, height: Int, pixels: inout [UInt8]) -> CGContext {
    guard let value = CGContext(
        data: &pixels,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: width * 4,
        space: colorSpace,
        bitmapInfo: bitmapInfo
    ) else {
        fatalError("Could not create bitmap context")
    }
    value.interpolationQuality = .high
    return value
}

func cgImage(pixels: [UInt8], width: Int, height: Int) -> CGImage {
    let data = Data(pixels) as CFData
    guard
        let provider = CGDataProvider(data: data),
        let image = CGImage(
            width: width,
            height: height,
            bitsPerComponent: 8,
            bitsPerPixel: 32,
            bytesPerRow: width * 4,
            space: colorSpace,
            bitmapInfo: CGBitmapInfo(rawValue: bitmapInfo),
            provider: provider,
            decode: nil,
            shouldInterpolate: true,
            intent: .defaultIntent
        )
    else {
        fatalError("Could not create CGImage")
    }
    return image
}

func writePNG(_ image: CGImage, to url: URL) {
    guard let destination = CGImageDestinationCreateWithURL(
        url as CFURL,
        UTType.png.identifier as CFString,
        1,
        nil
    ) else {
        fatalError("Could not create PNG destination for \(url.path)")
    }
    CGImageDestinationAddImage(destination, image, nil)
    guard CGImageDestinationFinalize(destination) else {
        fatalError("Could not write \(url.path)")
    }
}

var sourcePixels = [UInt8](repeating: 0, count: sourceWidth * sourceHeight * 4)
let sourceContext = context(width: sourceWidth, height: sourceHeight, pixels: &sourcePixels)
sourceContext.draw(
    sourceImage,
    in: CGRect(x: 0, y: 0, width: sourceWidth, height: sourceHeight)
)

// The supplied artwork is a JPEG composited over white. Reconstruct a smooth
// alpha edge while unmatting the white fringe so the wordmark remains clean on
// Kotha's light and dark surfaces.
var extractedPixels = [UInt8](repeating: 0, count: sourcePixels.count)
var foregroundBounds = Bounds(
    minX: sourceWidth,
    minY: sourceHeight,
    maxX: 0,
    maxY: 0
)

for y in 0..<sourceHeight {
    for x in 0..<sourceWidth {
        let index = (y * sourceWidth + x) * 4
        let red = Double(sourcePixels[index])
        let green = Double(sourcePixels[index + 1])
        let blue = Double(sourcePixels[index + 2])
        let distance = max(255 - red, 255 - green, 255 - blue)
        let alpha = max(0, min(1, (distance - 14) / 96))

        guard alpha > 0 else { continue }

        let unmatte: (Double) -> UInt8 = { channel in
            let value = (channel - 255 * (1 - alpha)) / alpha
            return UInt8(max(0, min(255, Int(value.rounded()))))
        }

        extractedPixels[index] = unmatte(red)
        extractedPixels[index + 1] = unmatte(green)
        extractedPixels[index + 2] = unmatte(blue)
        extractedPixels[index + 3] = UInt8((alpha * 255).rounded())

        if alpha > 0.04 {
            foregroundBounds.include(x: x, y: y)
        }
    }
}

func crop(
    pixels: [UInt8],
    sourceWidth: Int,
    sourceHeight: Int,
    bounds: Bounds,
    margin: Int,
    keepLabels: Set<Int32>? = nil,
    labels: [Int32]? = nil
) -> (pixels: [UInt8], width: Int, height: Int) {
    let minX = max(0, bounds.minX - margin)
    let minY = max(0, bounds.minY - margin)
    let maxX = min(sourceWidth - 1, bounds.maxX + margin)
    let maxY = min(sourceHeight - 1, bounds.maxY + margin)
    let width = maxX - minX + 1
    let height = maxY - minY + 1
    var output = [UInt8](repeating: 0, count: width * height * 4)

    for y in 0..<height {
        for x in 0..<width {
            let sourceX = minX + x
            let sourceY = minY + y
            let sourceIndex = (sourceY * sourceWidth + sourceX) * 4
            let outputIndex = (y * width + x) * 4

            if let keepLabels, let labels {
                let label = labels[sourceY * sourceWidth + sourceX]
                if !keepLabels.contains(label) { continue }
            }

            output[outputIndex] = pixels[sourceIndex]
            output[outputIndex + 1] = pixels[sourceIndex + 1]
            output[outputIndex + 2] = pixels[sourceIndex + 2]
            output[outputIndex + 3] = pixels[sourceIndex + 3]
        }
    }

    return (output, width, height)
}

let wordmark = crop(
    pixels: extractedPixels,
    sourceWidth: sourceWidth,
    sourceHeight: sourceHeight,
    bounds: foregroundBounds,
    margin: 20
)
let wordmarkImage = cgImage(
    pixels: wordmark.pixels,
    width: wordmark.width,
    height: wordmark.height
)
writePNG(wordmarkImage, to: publicBrandURL.appendingPathComponent("kotha-wordmark.png"))

// Isolate connected shapes so the combined calligraphic wave/K can serve as a
// mark without dragging the rest of the wordmark into small icon contexts.
var labels = [Int32](repeating: -1, count: sourceWidth * sourceHeight)
var components: [Component] = []
let neighborOffsets = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0), (1, 0),
    (-1, 1), (0, 1), (1, 1),
]

for y in 0..<sourceHeight {
    for x in 0..<sourceWidth {
        let pixelIndex = y * sourceWidth + x
        if labels[pixelIndex] != -1 { continue }
        let alpha = extractedPixels[pixelIndex * 4 + 3]
        if alpha < 96 {
            labels[pixelIndex] = -2
            continue
        }

        let id = Int32(components.count)
        var queue = [pixelIndex]
        labels[pixelIndex] = id
        var cursor = 0
        var bounds = Bounds(minX: x, minY: y, maxX: x, maxY: y)

        while cursor < queue.count {
            let current = queue[cursor]
            cursor += 1
            let currentX = current % sourceWidth
            let currentY = current / sourceWidth
            bounds.include(x: currentX, y: currentY)

            for (offsetX, offsetY) in neighborOffsets {
                let nextX = currentX + offsetX
                let nextY = currentY + offsetY
                if nextX < 0 || nextX >= sourceWidth || nextY < 0 || nextY >= sourceHeight {
                    continue
                }
                let next = nextY * sourceWidth + nextX
                if labels[next] != -1 { continue }
                if extractedPixels[next * 4 + 3] < 96 {
                    labels[next] = -2
                    continue
                }
                labels[next] = id
                queue.append(next)
            }
        }

        components.append(Component(id: id, count: queue.count, bounds: bounds))
    }
}

guard let primaryMark = components
    .filter({ $0.bounds.minX < foregroundBounds.minX + foregroundBounds.width / 2 })
    .max(by: { $0.count < $1.count })
else {
    fatalError("Could not isolate the Kotha mark")
}

var keptComponentIds: Set<Int32> = [primaryMark.id]
var markBounds = primaryMark.bounds

for component in components {
    let centerX = component.bounds.minX + component.bounds.width / 2
    let isAccentSized = component.count > 120 && component.count < primaryMark.count / 7
    let isNearLowerRight =
        centerX > primaryMark.bounds.minX + primaryMark.bounds.width * 2 / 3 &&
        centerX < primaryMark.bounds.maxX + primaryMark.bounds.width / 4 &&
        component.bounds.minY > primaryMark.bounds.minY + primaryMark.bounds.height / 2

    if isAccentSized && isNearLowerRight {
        keptComponentIds.insert(component.id)
        markBounds.include(x: component.bounds.minX, y: component.bounds.minY)
        markBounds.include(x: component.bounds.maxX, y: component.bounds.maxY)
    }
}

let mark = crop(
    pixels: extractedPixels,
    sourceWidth: sourceWidth,
    sourceHeight: sourceHeight,
    bounds: markBounds,
    margin: 20,
    keepLabels: keptComponentIds,
    labels: labels
)
let markImage = cgImage(pixels: mark.pixels, width: mark.width, height: mark.height)
writePNG(markImage, to: publicBrandURL.appendingPathComponent("kotha-mark.png"))

// A tighter crop around the K itself stays legible as a 24px navigation glyph.
let symbolStartX = Int(Double(mark.width) * 0.48)
let symbolBounds = Bounds(
    minX: symbolStartX,
    minY: 0,
    maxX: mark.width - 1,
    maxY: mark.height - 1
)
let symbol = crop(
    pixels: mark.pixels,
    sourceWidth: mark.width,
    sourceHeight: mark.height,
    bounds: symbolBounds,
    margin: 0
)
let symbolImage = cgImage(pixels: symbol.pixels, width: symbol.width, height: symbol.height)
writePNG(symbolImage, to: publicBrandURL.appendingPathComponent("kotha-symbol.png"))

func render(width: Int, height: Int, draw: (CGContext) -> Void) -> CGImage {
    var pixels = [UInt8](repeating: 0, count: width * height * 4)
    let value = context(width: width, height: height, pixels: &pixels)
    draw(value)
    guard let image = value.makeImage() else { fatalError("Could not render asset") }
    return image
}

func aspectFit(_ image: CGImage, in bounds: CGRect, inset: CGFloat = 0) -> CGRect {
    let available = bounds.insetBy(dx: inset, dy: inset)
    let scale = min(
        available.width / CGFloat(image.width),
        available.height / CGFloat(image.height)
    )
    let width = CGFloat(image.width) * scale
    let height = CGFloat(image.height) * scale
    return CGRect(
        x: available.midX - width / 2,
        y: available.midY - height / 2,
        width: width,
        height: height
    )
}

let appIcon = render(width: 1024, height: 1024) { value in
    value.setFillColor(CGColor(red: 0.957, green: 0.925, blue: 0.867, alpha: 1))
    value.fill(CGRect(x: 0, y: 0, width: 1024, height: 1024))
    value.draw(markImage, in: aspectFit(markImage, in: CGRect(x: 0, y: 0, width: 1024, height: 1024), inset: 68))
}
writePNG(appIcon, to: iconURL.appendingPathComponent("kotha-icon-master.png"))
writePNG(appIcon, to: iconURL.appendingPathComponent("logo.png"))

func tintedSymbol(color: CGColor, canvas: Int = 64, inset: CGFloat = 8) -> CGImage {
    render(width: canvas, height: canvas) { value in
        let rect = aspectFit(symbolImage, in: CGRect(x: 0, y: 0, width: canvas, height: canvas), inset: inset)
        value.draw(symbolImage, in: rect)
        value.setBlendMode(.sourceIn)
        value.setFillColor(color)
        value.fill(CGRect(x: 0, y: 0, width: canvas, height: canvas))
        value.setBlendMode(.normal)
    }
}

enum TrayState {
    case idle
    case recording
    case transcribing
    case warning
}

func trayAsset(color: CGColor?, state: TrayState, colored: Bool) -> CGImage {
    render(width: 64, height: 64) { value in
        let base = colored
            ? symbolImage
            : tintedSymbol(color: color ?? CGColor(gray: 1, alpha: 1))
        let symbolRect = state == .idle
            ? CGRect(x: 7, y: 7, width: 50, height: 50)
            : CGRect(x: 5, y: 10, width: 44, height: 44)
        value.draw(base, in: aspectFit(base, in: symbolRect))

        let badgeColor = color ?? CGColor(red: 0.71, green: 0.18, blue: 0.12, alpha: 1)
        value.setFillColor(badgeColor)

        switch state {
        case .idle:
            break
        case .recording:
            value.fillEllipse(in: CGRect(x: 45, y: 7, width: 14, height: 14))
        case .transcribing:
            value.fill(CGRect(x: 45, y: 9, width: 4, height: 9))
            value.fill(CGRect(x: 51, y: 6, width: 4, height: 15))
            value.fill(CGRect(x: 57, y: 11, width: 4, height: 7))
        case .warning:
            value.fillEllipse(in: CGRect(x: 43, y: 5, width: 18, height: 18))
            value.setBlendMode(.clear)
            value.fill(CGRect(x: 51, y: 10, width: 2, height: 7))
            value.fillEllipse(in: CGRect(x: 51, y: 8, width: 2, height: 2))
            value.setBlendMode(.normal)
        }
    }
}

let lightGlyph = CGColor(gray: 1, alpha: 0.94)
let darkGlyph = CGColor(red: 0.10, green: 0.12, blue: 0.10, alpha: 1)

let trayAssets: [(String, CGImage)] = [
    ("tray_idle.png", trayAsset(color: lightGlyph, state: .idle, colored: false)),
    ("tray_recording.png", trayAsset(color: lightGlyph, state: .recording, colored: false)),
    ("tray_transcribing.png", trayAsset(color: lightGlyph, state: .transcribing, colored: false)),
    ("tray_idle_warning.png", trayAsset(color: lightGlyph, state: .warning, colored: false)),
    ("tray_idle_dark.png", trayAsset(color: darkGlyph, state: .idle, colored: false)),
    ("tray_recording_dark.png", trayAsset(color: darkGlyph, state: .recording, colored: false)),
    ("tray_transcribing_dark.png", trayAsset(color: darkGlyph, state: .transcribing, colored: false)),
    ("tray_idle_warning_dark.png", trayAsset(color: darkGlyph, state: .warning, colored: false)),
    ("handy.png", trayAsset(color: nil, state: .idle, colored: true)),
    ("recording.png", trayAsset(color: nil, state: .recording, colored: true)),
    ("transcribing.png", trayAsset(color: nil, state: .transcribing, colored: true)),
    ("handy_warning.png", trayAsset(color: nil, state: .warning, colored: true)),
]

for (name, image) in trayAssets {
    writePNG(image, to: trayURL.appendingPathComponent(name))
}

print("Generated Kotha wordmark, mark, symbol, app-icon master, and tray states.")
print("Primary mark component: \(primaryMark.count) px, \(primaryMark.bounds.width)x\(primaryMark.bounds.height)")
