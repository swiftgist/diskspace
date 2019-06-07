$OUTPUT_NAME = "diskspace-0.3.0"
mkdir -Force $OUTPUT_NAME 
powershell copy-item -path target/release/ds.exe -destination $OUTPUT_NAME
powershell copy-item -path README.md -destination $OUTPUT_NAME
powershell copy-item -path LICENSE -destination $OUTPUT_NAME
dir $OUTPUT_NAME
$ZIPFILE = $OUTPUT_NAME + ".zip"
Compress-Archive -Update -Path $OUTPUT_NAME/* -DestinationPath $ZIPFILE
