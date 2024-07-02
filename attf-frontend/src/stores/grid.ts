import { defineStore } from 'pinia'
import { reactive } from 'vue'


function onmessage(event: any, ws: WebSocket) {
  const messageObj = JSON.parse(event.data)
  const grid = useGridStore()

  grid.entities = {}
  grid.entities_by_id = {}
  for (let idx in messageObj.entities) {
    let entity = messageObj.entities[idx].entity
    let resources: Resource[] = messageObj.entities[idx].resources
    if (!(entity.x in grid.entities)) {
      grid.entities[entity.x] = {}
    }
    let ent_obj = new Entity(entity.scenario_player_id, entity.game_class, entity.id, {});
    if (ent_obj.id == messageObj.playing) {
      grid.curX = entity.x
      grid.curY = entity.y
    }
    grid.entities[entity.x][entity.y] = ent_obj
    grid.entities_by_id[entity.id] = ent_obj
  }
  grid.grid = {}
  for (let idx in messageObj.visible_tiles) {
    let tile = messageObj.visible_tiles[idx]
    if (!(tile.x in grid.grid)) {
      grid.grid[tile.x] = {}
    }
    grid.grid[tile.x][tile.y] = { "type": tile.tile_type, "visible": true };
  }
  for (let idx in messageObj.allied_vision) {
    let tile = messageObj.allied_vision[idx]
    if (!(tile.x in grid.grid)) {
      grid.grid[tile.x] = {}
    }
    grid.grid[tile.x][tile.y] = { "type": tile.tile_type, "visible": false };
  }
  grid.isPlaying = true
  grid.actionLog = messageObj.logs
  grid.abilities = messageObj.abilities
  grid.ability = null
  grid.playing = messageObj.playing
  grid.user = grid.entities_by_id[messageObj.playing].owner
  let mouse = useMouseInfoStore()
  mouse.tileSelected = null
  ws.send("OK");
};

export const useMouseInfoStore = defineStore('mouseinfo', {
  state() {
    var tileDragged: Coords | null = null
    var tileSelected: Coords | null = null
    var mode: string = "normal"
    var onHighlight = () => { }
    return { tileDragged, tileSelected, mode, onHighlight }
  },
  getters: {
    draggedCreature(state) {
      if (state.tileDragged === null) return null
      const gridStore = useGridStore()
      return (gridStore.entities[state.tileDragged.x] || [])[state.tileDragged.y]
    },
    clickSelectedCreature(state) {
      if (state.tileSelected === null) return null
      const gridStore = useGridStore()
      return (gridStore.entities[state.tileSelected.x] || [])[state.tileSelected.y]
    },
  },
  actions: {
    clear() {
      this.tileSelected = null
      useGridStore().ability = null
    }
  }
})

type Resource = {
  resource_name: string;
}


export class Entity {
  owner: string
  game_class: string
  id: number
  resources: Object
  constructor(owner: string, game_class: string, id: number, resources: Object) {
    this.game_class = game_class
    this.id = id
    this.owner = owner
    this.resources = resources
  }
}

export class Coords {
  x: number
  y: number
  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }
}

export const useGridStore = defineStore('grid', {
  state: () => {
    let grid: { [x: number]: { [y: number]: Object } } = reactive({})
    let entities: { [x: number]: { [y: number]: Entity } } = reactive({})
    let entities_by_id: { [id: number]: Entity } = reactive({})
    let playing: number | null = null
    let user: string = ""
    let actionLog: any[] = reactive([])
    let abilities: string[] = reactive([])
    let ability: null | Object = null
    let curX: number | null = null
    let curY: number | null = null
    let gameId: number | null = null
    //var ws = new WebSocket('wss://' + location.host + "/api/" + location.pathname);


    const isPlaying = false

    return { grid, entities, playing, isPlaying, user, actionLog, curX, curY, entities_by_id, abilities, ability, gameId }
  },
  getters: {
    flat_grid(state) {
      var flat_grid = []
      for (let x in state.grid) {
        for (let y in state.grid[x]) {
          flat_grid.push([parseInt(x), parseInt(y), state.grid[x][y]])
        }
      }
      return flat_grid
    },
    proposedAbilities(state) {
      let abilities = []
      let mouse = useMouseInfoStore()
      if (!mouse.tileSelected) {
        return abilities
      }
      for (let ab in state.abilities) {
        for (let t in state.abilities[ab].targets) {
          if (state.abilities[ab].targets[t].x == mouse.tileSelected.x && state.abilities[ab].targets[t].y == mouse.tileSelected.y)
            abilities.push(parseInt(ab))
        }
      }
      return abilities
    },
  },
  actions: {
    connect(id: number) {
      var ws = new WebSocket('ws://' + location.host + '/game/' + id + '/ws');
      ws.onmessage = (event) => onmessage(event, ws)
      this.gameId = id
    },
    useAbility(x: number, y: number) {
      fetch('/game/' + this.gameId + '/ability/' + this.ability.name, { method: 'POST', headers: { 'Content-type': 'application/json' }, body: JSON.stringify({ x, y }) })
    }
  },
});

