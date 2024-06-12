# ruhr
A command line world clock

I have friends, family, and co-workers all over the globe. Totally sick of forgetting their time zone and googling the time in Istanbul or Berlin, I wanted to be able to easily get the time anywhere in the world from my command line.

It's as easy as `$ ruhr berlin`

The first time you run `ruhr`, it will send get the geospatial data from https://nominatim.openstreetmap.org/ui/search.html and then get the timezone using `tzf-rs` using the latitude and longitude data. The name of the place along with the timezone is then cached in an sqlite database. Easy peasy. 

If you run the command without any extra arguments, you will currently get only the top result back from open street maps. You might need to add differentiating information:

`$ ruhr 'berlin new jersey'` 

Will return the time in Berlin, New Jersey, USA -- otherwise, it will be Berlin, Germany.

Currently, you can add the command `--verbose` to and receive <detailed place name> <date> <time> <time zone abbreviation>:

```
$ ruhr berlin -v

$ Berlin, Germany 2024-06-12 05:50:31 CEST
```

## Installation
You can install using `$ cargo install ruhr`

## Contributing
I'd be happy to accept contributions, there are quite a few features I would like to add so if you find this tool useful please reach out!
