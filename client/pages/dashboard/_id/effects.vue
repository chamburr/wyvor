<template>
  <div>
    <BaseSelect
      v-model="preset"
      class="mb-4"
      placeholder="Select preset..."
      :allow-empty="false"
      :options="presetOptions"
    />
    <div v-for="(element, index) in effects" :key="index">
      <h2 v-if="element.key == null" class="mb-4 mt-5">
        {{ element.name }}
      </h2>
      <div v-else class="d-flex mb-4 align-items-center">
        <p class="effects-label mb-0">{{ element.name }}</p>
        <BaseSlider
          v-model.number="newFilters[getParent(index).toLowerCase()][element.key]"
          class="flex-grow-1"
          :range="{ min: parseFloat(element.minimum), max: parseFloat(element.maximum) }"
          @input="updatePreset"
        />
      </div>
    </div>
    <UpdateBar
      :changed="hasChanges"
      @save="saveFilters"
      @reset="newFilters = $api.clone(filters)"
    />
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'dashboard',
  async asyncData({ $getContent }) {
    return {
      effects: await $getContent('effects'),
    }
  },
  data() {
    return {
      newFilters: this.$api.clone(this.$store.getters['guild/player'].filters),
      preset: 'Default',
      presetOptions: ['Default', 'Custom'],
    }
  },
  computed: {
    hasChanges() {
      return !this.$api.isEqual(this.filters, this.newFilters)
    },
    filters() {
      return this.player.filters
    },
    ...mapGetters('guild', ['player']),
  },
  watch: {
    preset(newValue) {
      if (newValue !== 'Default') return
      for (const [index, element] of this.effects.entries()) {
        if (!element.key) continue
        this.newFilters[this.getParent(index).toLowerCase()][element.key] = element.default
      }
    },
  },
  mounted() {
    this.updatePreset()
  },
  methods: {
    updatePreset() {
      for (const [index, element] of this.effects.entries()) {
        if (
          element.key &&
          parseFloat(this.newFilters[this.getParent(index).toLowerCase()][element.key]) !==
            parseFloat(element.default)
        ) {
          this.preset = 'Custom'
          return
        }
      }
      this.preset = 'Default'
    },
    getParent(index) {
      return this.effects
        .slice(0, index)
        .reverse()
        .find(element => !element.key).name
    },
    async saveFilters() {
      const diff = this.$api.diff(this.filters, this.newFilters)
      await this.$axios
        .$patch(`/guilds/${this.$route.params.id}/player`, { filters: { ...diff } })
        .then(() => this.$toast.success('Updated the effects.'))
        .catch(this.$error)
    },
  },
}
</script>

<style scoped lang="scss">
.effects-label {
  width: 6em;
}
</style>
