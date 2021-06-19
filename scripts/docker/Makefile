nftmart: substrate staging_spec_raw.json
	docker build -f nftmart.Dockerfile -t btwiuse/nftmart:beta .
	docker push btwiuse/nftmart:beta

substrate:
	# please manually download it
	cp ../../target/release/substrate .

staging_spec_raw.json:
	# cp ../../bin/node/cli/res/staging_spec_raw.json .
	cp ../nftmart-keygen-example/staging_spec_raw.json .

clean:
	rm substrate staging_spec_raw.json
