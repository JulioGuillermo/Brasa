"""Follows the MPRIS players (playerctl) and keeps one current: the last one that started playing."""
import gi

gi.require_version("Playerctl", "2.0")
from gi.repository import GLib, Playerctl

PLAYING = Playerctl.PlaybackStatus.PLAYING


def metadata(player) -> dict:
    try:
        return player.props.metadata.unpack() or {}
    except (AttributeError, GLib.Error):
        return {}


def track_of(player) -> tuple:
    data = metadata(player)
    return data.get("xesam:title", ""), tuple(data.get("xesam:artist", []))


class Players:
    """on_change(reason) is called whenever what is shown should change: reason is "track", "status",
    "player" (another player became current) or "details" (same track, other metadata)."""

    def __init__(self, on_change):
        self.on_change = on_change
        self.current = None
        self.last_track = None
        self.last_status = None
        self.manager = Playerctl.PlayerManager()
        self.manager.connect("name-appeared", self.on_name_appeared)
        self.manager.connect("player-vanished", self.on_player_vanished)

    def start(self):
        for name in self.manager.props.player_names:
            self.on_name_appeared(self.manager, name)

    def on_name_appeared(self, manager, name):
        try:
            player = Playerctl.Player.new_from_name(name)
        except GLib.Error:
            return
        player.connect("playback-status", self.on_status)
        player.connect("metadata", self.on_metadata)
        manager.manage_player(player)
        if self.current is None or player.props.playback_status == PLAYING:
            self.make_current(player)

    def on_player_vanished(self, manager, player):
        if player is not self.current:
            return
        remaining = list(manager.props.players)
        playing = [p for p in remaining if p.props.playback_status == PLAYING]
        self.make_current((playing or remaining or [None])[0])

    def make_current(self, player):
        self.current = player
        self.last_track = track_of(player) if player else None
        self.last_status = player.props.playback_status if player else None
        self.on_change("player")

    def on_status(self, player, status):
        if status == PLAYING and player is not self.current:
            self.make_current(player)
            return
        if player is not self.current or status == self.last_status:
            return
        self.last_status = status
        self.on_change("status")

    def on_metadata(self, player, _metadata):
        if player is not self.current:
            return
        track = track_of(player)
        if track == self.last_track:
            self.on_change("details")
            return
        self.last_track = track
        self.on_change("track")
