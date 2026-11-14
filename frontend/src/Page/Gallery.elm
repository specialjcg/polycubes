module Page.Gallery exposing (Model, Msg, init, start, subscriptions, update, view)

{-| What typical polycubes look like. Two samples of the same size are
compared by how full their bounding boxes are: uniform draws against
polycubes grown by random neighbour. Some of the uniform draws are shown,
from the most compact to the least.
-}

import Backend exposing (Gallery, GrownPolycube)
import CellCountSlider
import Compactness
import Html exposing (Html, button, div, h2, p, section, span, text)
import Html.Attributes as Attr
import Html.Events exposing (onClick)
import Scene


{-| Must match `MAX_GALLERY_CELL_COUNT` of the backend.
-}
largestSampledCellCount : Int
largestSampledCellCount =
    10


initialCellCount : Int
initialCellCount =
    8


type alias Model =
    { cellCount : Int
    , gallery : Sampling
    }


type Sampling
    = Loading
    | Sampled Gallery
    | Failed String


type Msg
    = CellCountChosen String
    | GalleryRequested
    | GalleryFinished (Result String Gallery)


init : Model
init =
    { cellCount = initialCellCount, gallery = Loading }


{-| First gallery, to request once the backend listens.
-}
start : Model -> ( Model, Cmd Msg )
start model =
    requestGallery model.cellCount


requestGallery : Int -> ( Model, Cmd Msg )
requestGallery cellCount =
    ( { cellCount = cellCount, gallery = Loading }
    , Backend.requestGallery cellCount
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        CellCountChosen sliderValue ->
            requestGallery (CellCountSlider.parse sliderValue |> Maybe.withDefault model.cellCount)

        GalleryRequested ->
            requestGallery model.cellCount

        GalleryFinished (Ok gallery) ->
            ( { model | gallery = Sampled gallery }, Cmd.none )

        GalleryFinished (Err reason) ->
            ( { model | gallery = Failed reason }, Cmd.none )


subscriptions : Sub Msg
subscriptions =
    Backend.galleryFinished GalleryFinished


view : Model -> Html Msg
view model =
    div []
        [ div [ Attr.class "controls" ]
            [ CellCountSlider.view largestSampledCellCount CellCountChosen model.cellCount
            , button [ onClick GalleryRequested ] [ text "Nouveaux tirages" ]
            ]
        , viewSampling model.gallery
        ]


viewSampling : Sampling -> Html Msg
viewSampling sampling =
    case sampling of
        Loading ->
            p [ Attr.class "status" ] [ text "Tirages en cours…" ]

        Sampled gallery ->
            div []
                [ p [ Attr.class "explanation" ]
                    [ text
                        (String.fromInt gallery.sampleCount
                            ++ " polycubes par procédé, rangés selon le remplissage de leur boîte englobante : plus la boîte est pleine, plus la forme est compacte."
                        )
                    ]
                , div [ Attr.class "histograms" ]
                    [ viewHistogram "uniform" "Tirage uniforme" gallery.uniformHistogram
                    , viewHistogram "growth" "Croissance par voisin au hasard" gallery.growthHistogram
                    ]
                , h2 [] [ text "Tirages uniformes, du plus compact au moins compact" ]
                , div [ Attr.class "gallery" ] (List.map viewShownPolycube gallery.shown)
                ]

        Failed reason ->
            p [ Attr.class "status error" ] [ text ("Tirages impossibles : " ++ reason) ]


{-| Bars share the scale of their own histogram: the two samples are
compared by shape, their sizes being equal.
-}
viewHistogram : String -> String -> List Int -> Html Msg
viewHistogram className title polycubeCounts =
    let
        tallestCount =
            List.maximum polycubeCounts |> Maybe.withDefault 0 |> max 1

        viewBar polycubeCount =
            div
                [ Attr.class "bar"
                , Attr.style "height" (String.fromFloat (100 * toFloat polycubeCount / toFloat tallestCount) ++ "%")
                , Attr.title (String.fromInt polycubeCount ++ " polycubes")
                ]
                []
    in
    section []
        [ h2 [] [ text title ]
        , div [ Attr.class ("histogram " ++ className) ] (List.map viewBar polycubeCounts)
        , div [ Attr.class "histogram-axis" ]
            (List.map (\label -> span [] [ text label ]) [ "boîte vide", "remplie à 50 %", "boîte pleine" ])
        ]


viewShownPolycube : GrownPolycube -> Html Msg
viewShownPolycube polycube =
    section []
        [ Scene.view polycube.cells
        , p [ Attr.class "status" ] [ text (Compactness.describe polycube) ]
        ]
