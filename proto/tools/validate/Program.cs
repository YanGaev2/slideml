// Validates a .pptx with the official Open XML SDK validator.
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

using var doc = PresentationDocument.Open(args[0], false);
var errors = new OpenXmlValidator(FileFormatVersions.Microsoft365).Validate(doc).ToList();
foreach (var e in errors)
    Console.WriteLine($"{e.Part?.Uri} {e.Path?.XPath}\n  {e.Description}");
Console.WriteLine($"validator errors: {errors.Count}");
return errors.Count == 0 ? 0 : 1;
