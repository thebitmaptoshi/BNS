zmakin.gitbook.io/bns

Please read my Bitmap Naming System theory at the above address to understand they why and how of this code repository.

This repository is broken up into 3 branches. This main branch is how I started. It is the code i set up to run a bitmap indexer, prior to any implementation of the ordinals rust code into integration. This first open section would be just the base of the code and what i want it to accompplish. This would require running Bitcoin Core and ORD clients each up to date with their respective sections.

The second branch labeled "bns overlay" is a standalone overlay of the bitmap indexer built using the ord rust code as a large denepndency of the code, but not as a direct addition to the ord client. This is most likely the first route to test and start indexing bitmap, and thus prepare for a public BNS blockheight to be set

The third branch labeled "integration" is the code and setup on how we would go about adding the BI to the ord client as integrated code. A soft fork of ORD if you will, that adds bitmap and BNS registry as an optional field for anyone to activate in their ORD client. It would not be a required addition to any node operator's database, but would be integrated to allow for the robust addition that the bitmapping address system offers. Let's recreate the free DNS like it was meant to  be Baby!
