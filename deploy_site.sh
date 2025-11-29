#!/bin/bash

set -e -u

cd site/root

OUTPUT_BUCKET_NAME=dpedal.com

# TODO: cache-control

echo "Uploading assets/"
aws s3 sync assets s3://$OUTPUT_BUCKET_NAME/assets --cache-control no-cache --content-encoding gzip

echo "Uploading media/"
aws s3 sync media s3://$OUTPUT_BUCKET_NAME/media --cache-control no-cache --content-encoding none

echo "Uploading everything else"
aws s3 sync . s3://$OUTPUT_BUCKET_NAME --exclude "assets/*" --exclude "media/*" --cache-control no-cache --content-encoding gzip
