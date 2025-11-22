#!/bin/bash

set -e -u

cd site/root

OUTPUT_BUCKET_NAME=dpedal.com

# just deploy it all with no cache and no compression, we can come back to this later
aws s3 sync . s3://$OUTPUT_BUCKET_NAME --cache-control no-cache
