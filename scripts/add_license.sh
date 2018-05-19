#!/bin/sh

PAT="^// Copyright [0-9]{4}-[0-9]{4} Parity Technologies \(UK\) Ltd\.
// This file is part of Parity\.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// \(at your option\) any later version\.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE\.  See the
// GNU General Public License for more details\.

// You should have received a copy of the GNU General Public License
// along with Parity\.  If not, see <http://www.gnu.org/licenses/>\.$"

for f in $(find . -name '*.rs'); do
	HEADER=$(head -16 $f)
	if [[ $HEADER =~ $PAT ]]; then
		BODY=$(tail -n +17 $f)
		cat license_header > temp
		echo "$BODY" >> temp
		mv temp $f
	else
		echo "$f was missing header" 
		cat license_header $f > temp
		mv temp $f
	fi
done
